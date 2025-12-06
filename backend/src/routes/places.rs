use std::collections::VecDeque;

use axum::{
    extract::{multipart::Multipart, DefaultBodyLimit, Extension, Path as AxumPath, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use mime_guess::mime;
use tracing::error;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::jwt::JwtClaims;
use crate::repository::image_store::ImageUpload;
use crate::repository::place::{
    NewPlace, NewPlaceImage, PlaceRecord, PlaceRepository, PlaceRepositoryError, UpdatePlace,
};

use super::middleware::jwt_auth;
use super::models::{ErrorResponse, PlaceImageResponse, PlaceResponse};

// The default Axum body limit is 2MB, which is too small for typical phone photos.
const MAX_MULTIPART_SIZE_BYTES: usize = 25 * 1024 * 1024;

pub fn router(state: AppState) -> Router {
    let middleware_state = state.clone();

    Router::new()
        .route("/places", post(create_place).get(list_places))
        .route("/places/:id", get(get_place).patch(update_place))
        .route("/places/:id/images", get(list_images))
        .route("/places/:place_id/images/:image_id", get(get_place_image))
        .route_layer(middleware::from_fn_with_state(middleware_state, jwt_auth))
        .layer(DefaultBodyLimit::max(MAX_MULTIPART_SIZE_BYTES))
        .with_state(state)
}

#[derive(Default)]
struct IncomingPlace {
    id: Option<Uuid>,
    name: Option<String>,
    category: Option<String>,
    location: Option<String>,
    note: Option<String>,
    images: Vec<IncomingImage>,
}

struct IncomingImage {
    id: Option<Uuid>,
    file_name: Option<String>,
    bytes: Vec<u8>,
}

async fn create_place(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    mut multipart: Multipart,
) -> Result<Json<PlaceResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut form = IncomingPlace::default();
    let mut image_ids: VecDeque<Uuid> = VecDeque::new();

    while let Some(field) = multipart.next_field().await.map_err(|err| {
        error!(?err, "failed to read form-data field");
        internal_error()
    })? {
        let field_name = field.name().map(|value| value.to_owned());
        match field_name.as_deref() {
            Some("id") => {
                let text = field
                    .text()
                    .await
                    .map_err(|err| {
                        error!(?err, "invalid id field");
                        bad_request("id must be text")
                    })?
                    .trim()
                    .to_string();
                form.id = Some(parse_uuid(&text, "id")?);
            }
            Some("name") => {
                form.name = Some(
                    field
                        .text()
                        .await
                        .map_err(|err| {
                            error!(?err, "invalid name field");
                            bad_request("name must be text")
                        })?
                        .trim()
                        .to_string(),
                );
            }
            Some("category") => {
                form.category = Some(
                    field
                        .text()
                        .await
                        .map_err(|err| {
                            error!(?err, "invalid category field");
                            bad_request("category must be text")
                        })?
                        .trim()
                        .to_string(),
                );
            }
            Some("location") => {
                form.location = Some(
                    field
                        .text()
                        .await
                        .map_err(|err| {
                            error!(?err, "invalid location field");
                            bad_request("location must be text")
                        })?
                        .trim()
                        .to_string(),
                );
            }
            Some("note") => {
                form.note = Some(
                    field
                        .text()
                        .await
                        .map_err(|err| {
                            error!(?err, "invalid note field");
                            bad_request("note must be text")
                        })?
                        .trim()
                        .to_string(),
                );
            }
            Some("image") => {
                let file_name = field.file_name().map(|value| value.to_owned());
                let bytes = field.bytes().await.map_err(|err| {
                    error!(?err, "failed to read image bytes");
                    bad_request("image upload failed")
                })?;
                let image_id = image_ids
                    .pop_front()
                    .ok_or_else(|| missing_field("image_id before each image"))?;
                form.images.push(IncomingImage {
                    id: Some(image_id),
                    file_name,
                    bytes: bytes.to_vec(),
                });
            }
            Some("image_id") => {
                let text = field
                    .text()
                    .await
                    .map_err(|err| {
                        error!(?err, "invalid image_id field");
                        bad_request("image_id must be text")
                    })?
                    .trim()
                    .to_string();
                image_ids.push_back(parse_uuid(&text, "image_id")?);
            }
            _ => {
                // Ignore unknown fields to keep the API forward compatible.
            }
        }
    }

    if !image_ids.is_empty() {
        return Err(missing_field("image_id for every image"));
    }

    let place_id = form.id.ok_or_else(|| missing_field("id"))?;
    let name = form.name.ok_or_else(|| missing_field("name"))?;
    let category = form.category.ok_or_else(|| missing_field("category"))?;
    let location = form.location.ok_or_else(|| missing_field("location"))?;

    let repository = state.place_repository();
    let image_store = state.image_store();

    let uploads = prepare_uploads(form.images)?;
    let stored_images = image_store
        .save_images(place_id, uploads)
        .await
        .map_err(|err| {
            error!(?err, "failed to persist place images");
            image_io_error("could not store image file")
        })?;

    let new_place = NewPlace {
        id: place_id,
        user_id: claims.sub,
        name: &name,
        category: &category,
        location: &location,
        note: form.note.as_deref(),
    };

    let image_payloads: Vec<NewPlaceImage<'_>> = stored_images
        .iter()
        .map(|stored| NewPlaceImage {
            id: stored.id,
            place_id,
            file_name: &stored.file_name,
            caption: None,
        })
        .collect();

    let (record, inserted_images) = match repository
        .create_place_with_images(new_place, &image_payloads)
        .await
    {
        Ok(result) => result,
        Err(err) => {
            error!(?err, "failed to create place");
            image_store.cleanup_images(place_id, &stored_images).await;
            return Err(internal_error());
        }
    };

    Ok(Json(enrich_place(
        record,
        inserted_images
            .into_iter()
            .map(PlaceImageResponse::from_record)
            .collect(),
    )))
}

async fn list_places(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
) -> Result<Json<Vec<PlaceResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let repository = state.place_repository();
    let places = repository.list_for_user(claims.sub).await.map_err(|err| {
        error!(?err, "failed to list places");
        internal_error()
    })?;

    let mut responses = Vec::with_capacity(places.len());
    for place in places {
        let images = load_images_for_place(&repository, claims.sub, place.id)
            .await
            .map_err(|err| {
                error!(?err, "failed to load images for place");
                internal_error()
            })?;
        responses.push(enrich_place(place, images));
    }

    Ok(Json(responses))
}

async fn get_place(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    AxumPath(place_id): AxumPath<Uuid>,
) -> Result<Json<PlaceResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repository = state.place_repository();
    let place = repository
        .find_for_user(claims.sub, place_id)
        .await
        .map_err(|err| {
            error!(?err, "failed to load place");
            internal_error()
        })?
        .ok_or_else(place_not_found)?;

    let images = load_images_for_place(&repository, claims.sub, place.id)
        .await
        .map_err(|err| {
            error!(?err, "failed to load images for place");
            internal_error()
        })?;

    Ok(Json(enrich_place(place, images)))
}

async fn update_place(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    AxumPath(place_id): AxumPath<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<PlaceResponse>, (StatusCode, Json<ErrorResponse>)> {
    let repository = state.place_repository();

    // Verify place belongs to user.
    repository
        .find_for_user(claims.sub, place_id)
        .await
        .map_err(|err| {
            error!(?err, "failed to verify place");
            internal_error()
        })?
        .ok_or_else(place_not_found)?;

    let mut update = UpdatePlace::default();
    let mut incoming_images = Vec::new();
    let mut image_ids: VecDeque<Uuid> = VecDeque::new();
    let mut delete_image_ids: Vec<Uuid> = Vec::new();

    while let Some(field) = multipart.next_field().await.map_err(|err| {
        error!(?err, "failed to read form-data field");
        internal_error()
    })? {
        let Some(name) = field.name() else {
            continue;
        };

        match name {
            "name" => update.name = Some(read_text_field(field, "name").await?),
            "category" => update.category = Some(read_text_field(field, "category").await?),
            "location" => update.location = Some(read_text_field(field, "location").await?),
            "note" => update.note = Some(read_text_field(field, "note").await?),
            "image" => {
                let file_name = field.file_name().map(|value| value.to_owned());
                let bytes = field.bytes().await.map_err(|err| {
                    error!(?err, "failed to read image bytes");
                    bad_request("image upload failed")
                })?;
                let image_id = image_ids
                    .pop_front()
                    .ok_or_else(|| missing_field("image_id before each image"))?;
                incoming_images.push(IncomingImage {
                    id: Some(image_id),
                    file_name,
                    bytes: bytes.to_vec(),
                });
            }
            "image_id" => {
                let text = read_text_field(field, "image_id").await?;
                image_ids.push_back(parse_uuid(&text, "image_id")?);
            }
            "delete_image_ids" => {
                let text = read_text_field(field, "delete_image_ids").await?;
                delete_image_ids = serde_json::from_str::<Vec<String>>(&text)
                    .map_err(|err| {
                        error!(?err, "invalid delete_image_ids payload");
                        bad_request("delete_image_ids must be a JSON array of UUID strings")
                    })?
                    .into_iter()
                    .map(|raw| parse_uuid(&raw, "delete_image_ids"))
                    .collect::<Result<Vec<_>, _>>()?;
            }
            _ => {}
        }
    }

    if !image_ids.is_empty() {
        return Err(missing_field("image_id for every image"));
    }

    let image_store = state.image_store();
    let uploads = prepare_uploads(incoming_images)?;
    let stored_images = if uploads.is_empty() {
        Vec::new()
    } else {
        image_store
            .save_images(place_id, uploads)
            .await
            .map_err(|err| {
                error!(?err, "failed to persist images during update");
                image_io_error("could not store image file")
            })?
    };

    let new_image_payloads: Vec<NewPlaceImage<'_>> = stored_images
        .iter()
        .map(|stored| NewPlaceImage {
            id: stored.id,
            place_id,
            file_name: &stored.file_name,
            caption: None,
        })
        .collect();

    let (place, _inserted_images, deleted_images) = match repository
        .update_place_with_images(
            claims.sub,
            place_id,
            update,
            &new_image_payloads,
            &delete_image_ids,
        )
        .await
    {
        Ok(result) => result,
        Err(err) => {
            error!(?err, "failed to update place");
            image_store.cleanup_images(place_id, &stored_images).await;
            return Err(internal_error());
        }
    };

    let deleted_file_names: Vec<String> = deleted_images
        .iter()
        .map(|img| img.file_name.clone())
        .collect();
    image_store
        .remove_files(place_id, &deleted_file_names)
        .await;

    // Build response with current images.
    let images = load_images_for_place(&repository, claims.sub, place_id)
        .await
        .map_err(|err| {
            error!(?err, "failed to load images after update");
            internal_error()
        })?;

    Ok(Json(enrich_place(place, images)))
}

async fn list_images(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    AxumPath(place_id): AxumPath<Uuid>,
) -> Result<Json<Vec<PlaceImageResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let repository = state.place_repository();
    // Ensure place exists and images obey policy
    repository
        .find_for_user(claims.sub, place_id)
        .await
        .map_err(|err| {
            error!(?err, "failed to verify place");
            internal_error()
        })?
        .ok_or_else(place_not_found)?;

    let images = load_images_for_place(&repository, claims.sub, place_id)
        .await
        .map_err(|err| {
            error!(?err, "failed to load images");
            internal_error()
        })?;

    Ok(Json(images))
}

async fn get_place_image(
    State(state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    AxumPath((place_id, image_id)): AxumPath<(Uuid, Uuid)>,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let repository = state.place_repository();
    let image = repository
        .find_image_for_user(claims.sub, image_id)
        .await
        .map_err(|err| {
            error!(?err, "failed to load place");
            internal_error()
        })?
        .ok_or_else(place_not_found)?;

    if image.place_id != place_id {
        return Err(place_not_found());
    }

    let image_store = state.image_store();
    let bytes = image_store
        .get_image(place_id, &image.file_name)
        .await
        .map_err(|err| {
            error!(?err, "failed to read image from disk");
            image_io_error("could not read image file")
        })?;

    let mime = mime_guess::from_path(&image.file_name).first_or(mime::APPLICATION_OCTET_STREAM);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(mime.as_ref())
            .unwrap_or(HeaderValue::from_static("application/octet-stream")),
    );

    Ok((headers, bytes).into_response())
}

async fn load_images_for_place(
    repository: &PlaceRepository,
    user_id: Uuid,
    place_id: Uuid,
) -> Result<Vec<PlaceImageResponse>, PlaceRepositoryError> {
    let images = repository
        .list_images_for_place(user_id, place_id)
        .await?
        .into_iter()
        .map(PlaceImageResponse::from_record)
        .collect();
    Ok(images)
}

fn enrich_place(place: PlaceRecord, images: Vec<PlaceImageResponse>) -> PlaceResponse {
    let mut response = PlaceResponse::from(place);
    response.images = images;
    response
}

fn missing_field(field: &'static str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse::new(
            "invalid_request",
            format!("missing required field: {}", field),
        )),
    )
}

fn bad_request(message: &'static str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse::new("invalid_request", message)),
    )
}

async fn read_text_field(
    field: axum::extract::multipart::Field<'_>,
    label: &'static str,
) -> Result<String, (StatusCode, Json<ErrorResponse>)> {
    field
        .text()
        .await
        .map_err(|err| {
            error!(?err, "invalid {} field", label);
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(
                    "invalid_request",
                    format!("{} must be text", label),
                )),
            )
        })
        .map(|s| s.trim().to_string())
}

fn parse_uuid(value: &str, field: &'static str) -> Result<Uuid, (StatusCode, Json<ErrorResponse>)> {
    Uuid::parse_str(value).map_err(|err| {
        error!(%value, ?err, "invalid uuid");
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                "invalid_request",
                format!("{} must be a valid UUID", field),
            )),
        )
    })
}

fn image_io_error(message: &'static str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new("image_io_error", message)),
    )
}

fn place_not_found() -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse::new("not_found", "place not found")),
    )
}

fn internal_error() -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new(
            "internal_error",
            "unexpected server error",
        )),
    )
}

fn prepare_uploads(
    incoming: Vec<IncomingImage>,
) -> Result<Vec<ImageUpload>, (StatusCode, Json<ErrorResponse>)> {
    let mut uploads = Vec::new();
    for image in incoming {
        let image_id = image
            .id
            .ok_or_else(|| missing_field("image_id before each image"))?;
        uploads.push(ImageUpload {
            id: image_id,
            file_name: image.file_name,
            bytes: image.bytes,
        });
    }
    Ok(uploads)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{header, Request};
    use tower::ServiceExt;

    use crate::test_utils::router::{multipart_body, parse_json, Part, TestContext};

    #[tokio::test]
    async fn create_list_get_and_fetch_image() {
        let ctx = TestContext::new(super::router).await;
        let user = ctx.insert_user().await;
        let token = ctx.jwt.generate(&user).expect("jwt");

        let place_id = Uuid::new_v4();
        let image_id = Uuid::new_v4();
        let (boundary, body) = multipart_body(vec![
            Part::text("id", place_id.to_string()),
            Part::text("name", "Blue Bottle"),
            Part::text("category", "Coffee"),
            Part::text("location", "Oakland"),
            Part::text("note", "Try the latte"),
            Part::text("image_id", image_id.to_string()),
            Part::file("image", "image.jpg", "image/jpeg", b"IMG".to_vec()),
        ]);

        let response = ctx
            .app
            .clone()
            .oneshot(
                Request::post("/places")
                    .header("Authorization", format!("Bearer {}", token))
                    .header(
                        header::CONTENT_TYPE,
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .expect("request succeeds");
        assert_eq!(response.status(), StatusCode::OK);
        let place: PlaceResponse = parse_json(response).await;
        assert_eq!(place.id, place_id);
        assert_eq!(place.images.len(), 1);

        let list_response = ctx
            .app
            .clone()
            .oneshot(
                Request::get("/places")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("list request");
        assert_eq!(list_response.status(), StatusCode::OK);
        let places: Vec<PlaceResponse> = parse_json(list_response).await;
        assert_eq!(places.len(), 1);

        let get_response = ctx
            .app
            .clone()
            .oneshot(
                Request::get(format!("/places/{place_id}"))
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("get request");
        assert_eq!(get_response.status(), StatusCode::OK);

        let image_resp = ctx
            .app
            .clone()
            .oneshot(
                Request::get(format!("/places/{place_id}/images/{image_id}"))
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("image request");
        assert_eq!(image_resp.status(), StatusCode::OK);

        let file_name: String =
            sqlx::query_scalar("SELECT file_name FROM place_images WHERE id = $1")
                .bind(image_id)
                .fetch_one(&ctx.pool)
                .await
                .unwrap();
        let expected_path = ctx.image_dir().join(place_id.to_string()).join(&file_name);
        assert!(expected_path.exists(), "image file should exist");
    }

    #[tokio::test]
    async fn update_place_adds_and_deletes_images() {
        let ctx = TestContext::new(super::router).await;
        let user = ctx.insert_user().await;
        let token = ctx.jwt.generate(&user).expect("jwt");

        let place_id = Uuid::new_v4();
        let original_image_id = Uuid::new_v4();
        create_place_for_test(&ctx, &token, place_id, original_image_id).await;

        let original_file: String =
            sqlx::query_scalar("SELECT file_name FROM place_images WHERE id = $1")
                .bind(original_image_id)
                .fetch_one(&ctx.pool)
                .await
                .unwrap();

        let new_image_id = Uuid::new_v4();
        let delete_payload = serde_json::to_string(&vec![original_image_id.to_string()]).unwrap();
        let (boundary, body) = multipart_body(vec![
            Part::text("name", "Updated Name"),
            Part::text("delete_image_ids", delete_payload),
            Part::text("image_id", new_image_id.to_string()),
            Part::file("image", "new.jpg", "image/jpeg", b"NEW".to_vec()),
        ]);

        let response = ctx
            .app
            .clone()
            .oneshot(
                Request::patch(format!("/places/{place_id}"))
                    .header("Authorization", format!("Bearer {}", token))
                    .header(
                        header::CONTENT_TYPE,
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .expect("update request");
        assert_eq!(response.status(), StatusCode::OK);
        let updated: PlaceResponse = parse_json(response).await;
        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.images.len(), 1);
        assert_eq!(updated.images[0].id, new_image_id);

        let old_file: Option<String> =
            sqlx::query_scalar("SELECT file_name FROM place_images WHERE id = $1")
                .bind(original_image_id)
                .fetch_optional(&ctx.pool)
                .await
                .unwrap();
        assert!(old_file.is_none());

        let new_file: String =
            sqlx::query_scalar("SELECT file_name FROM place_images WHERE id = $1")
                .bind(new_image_id)
                .fetch_one(&ctx.pool)
                .await
                .unwrap();
        let new_path = ctx.image_dir().join(place_id.to_string()).join(&new_file);
        assert!(new_path.exists());

        let old_path = ctx
            .image_dir()
            .join(place_id.to_string())
            .join(&original_file);
        assert!(!old_path.exists());
    }

    #[tokio::test]
    async fn user_cannot_access_foreign_place_or_image() {
        let ctx = TestContext::new(super::router).await;
        let owner = ctx.insert_user().await;
        let owner_token = ctx.jwt.generate(&owner).expect("jwt");
        let intruder = ctx.insert_user().await;
        let intruder_token = ctx.jwt.generate(&intruder).expect("jwt");

        let place_id = Uuid::new_v4();
        let image_id = Uuid::new_v4();
        create_place_for_test(&ctx, &owner_token, place_id, image_id).await;

        let forbidden_place = ctx
            .app
            .clone()
            .oneshot(
                Request::get(format!("/places/{place_id}"))
                    .header("Authorization", format!("Bearer {}", intruder_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("intruder place request");
        assert_eq!(forbidden_place.status(), StatusCode::NOT_FOUND);

        let forbidden_image = ctx
            .app
            .clone()
            .oneshot(
                Request::get(format!("/places/{place_id}/images/{image_id}"))
                    .header("Authorization", format!("Bearer {}", intruder_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("intruder image request");
        assert_eq!(forbidden_image.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn create_place_rejects_missing_image_ids() {
        let ctx = TestContext::new(super::router).await;
        let user = ctx.insert_user().await;
        let token = ctx.jwt.generate(&user).expect("jwt");
        let place_id = Uuid::new_v4();

        let (boundary, body) = multipart_body(vec![
            Part::text("id", place_id.to_string()),
            Part::text("name", "No Image Id"),
            Part::text("category", "Test"),
            Part::text("location", "Nowhere"),
            Part::text("note", "bad"),
            Part::file("image", "bad.jpg", "image/jpeg", b"BYTES".to_vec()),
        ]);

        let response = ctx
            .app
            .clone()
            .oneshot(
                Request::post("/places")
                    .header("Authorization", format!("Bearer {}", token))
                    .header(
                        header::CONTENT_TYPE,
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .expect("request succeeds");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    async fn create_place_for_test(ctx: &TestContext, token: &str, place_id: Uuid, image_id: Uuid) {
        let (boundary, body) = multipart_body(vec![
            Part::text("id", place_id.to_string()),
            Part::text("name", "Sample"),
            Part::text("category", "Coffee"),
            Part::text("location", "Somewhere"),
            Part::text("image_id", image_id.to_string()),
            Part::file("image", "orig.jpg", "image/jpeg", vec![1, 2, 3]),
        ]);

        let response = ctx
            .app
            .clone()
            .oneshot(
                Request::post("/places")
                    .header("Authorization", format!("Bearer {}", token))
                    .header(
                        header::CONTENT_TYPE,
                        format!("multipart/form-data; boundary={boundary}"),
                    )
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .expect("create request");
        assert_eq!(response.status(), StatusCode::OK);
    }
}
