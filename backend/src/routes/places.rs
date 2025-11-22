use std::collections::VecDeque;
use std::path::{Path, PathBuf};

use axum::{
    extract::{multipart::Multipart, Extension, Path as AxumPath, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use mime_guess::mime;
use tokio::fs;
use tracing::error;
use uuid::Uuid;

use crate::app_state::AppState;
use crate::jwt::JwtClaims;
use crate::repository::place::{
    NewPlace, NewPlaceImage, PlaceRecord, PlaceRepository, PlaceRepositoryError, UpdatePlace,
};

use super::middleware::jwt_auth;
use super::models::{ErrorResponse, PlaceImageResponse, PlaceResponse};

pub fn router(state: AppState) -> Router {
    let middleware_state = state.clone();

    Router::new()
        .route("/places", post(create_place).get(list_places))
        .route("/places/:id", get(get_place).patch(update_place))
        .route("/places/:id/images", get(list_images))
        .route("/places/:place_id/images/:image_id", get(get_place_image))
        .route_layer(middleware::from_fn_with_state(middleware_state, jwt_auth))
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

    if image_ids.len() != form.images.len() {
        return Err(missing_field("image_id for every image"));
    }

    let place_id = form.id.ok_or_else(|| missing_field("id"))?;
    let name = form.name.ok_or_else(|| missing_field("name"))?;
    let category = form.category.ok_or_else(|| missing_field("category"))?;
    let location = form.location.ok_or_else(|| missing_field("location"))?;

    let image_dir = state.place_image_dir();

    let repository = state.place_repository();

    let stored_images = store_images_to_disk(place_id, &image_dir, form.images).await?;

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
            cleanup_files(&stored_images).await;
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

    if image_ids.len() != incoming_images.len() {
        return Err(missing_field("image_id for every image"));
    }

    let image_dir = state.place_image_dir();
    let stored_images = store_images_to_disk(place_id, &image_dir, incoming_images).await?;

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
            cleanup_files(&stored_images).await;
            return Err(internal_error());
        }
    };

    // Remove files for deleted images.
    for deleted in &deleted_images {
        let path = image_dir
            .join(place_id.to_string())
            .join(&deleted.file_name);
        if let Err(err) = fs::remove_file(&path).await {
            error!(?err, ?path, "failed to remove deleted image file");
        }
    }

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

    let image_dir = state.place_image_dir();
    let full_path = image_dir.join(Path::new(&format!(
        "{}/{}",
        image.place_id, image.file_name
    )));
    let bytes = fs::read(&full_path).await.map_err(|err| {
        error!(?err, ?full_path, "failed to read image from disk");
        image_io_error("could not read image file")
    })?;

    let mime = mime_guess::from_path(&full_path).first_or(mime::APPLICATION_OCTET_STREAM);

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

async fn cleanup_files(paths: &[StoredImage]) {
    for stored in paths {
        if let Err(err) = fs::remove_file(&stored.path).await {
            error!(?err, path = ?stored.path, "failed to cleanup image file after error");
        }
    }
}

fn enrich_place(place: PlaceRecord, images: Vec<PlaceImageResponse>) -> PlaceResponse {
    let mut response = PlaceResponse::from(place);
    response.images = images;
    response
}

async fn save_place_image(
    place_id: Uuid,
    image_id: Uuid,
    base_dir: &Path,
    file_name: Option<&str>,
    bytes: &[u8],
) -> Result<String, std::io::Error> {
    let place_dir = base_dir.join(place_id.to_string());
    fs::create_dir_all(&place_dir).await?;

    let extension = file_name
        .and_then(|name| Path::new(name).extension())
        .and_then(|ext| ext.to_str())
        .map(|ext| format!(".{}", ext));

    let stored_file_name = format!(
        "{}{}",
        image_id,
        extension.unwrap_or_else(|| String::from(""))
    );

    let full_path = place_dir.join(&stored_file_name);
    fs::write(full_path, bytes).await?;

    Ok(stored_file_name)
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

struct StoredImage {
    id: Uuid,
    file_name: String,
    path: PathBuf,
}

async fn store_images_to_disk(
    place_id: Uuid,
    base_dir: &PathBuf,
    incoming: Vec<IncomingImage>,
) -> Result<Vec<StoredImage>, (StatusCode, Json<ErrorResponse>)> {
    let mut stored = Vec::new();
    for image in incoming {
        let image_id = image
            .id
            .ok_or_else(|| missing_field("image_id before each image"))?;

        let stored_file_name = match save_place_image(
            place_id,
            image_id,
            base_dir.as_ref(),
            image.file_name.as_deref(),
            &image.bytes,
        )
        .await
        {
            Ok(name) => name,
            Err(err) => {
                error!(?err, "failed to persist place image");
                cleanup_files(&stored).await;
                return Err(image_io_error("could not store image file"));
            }
        };

        let path = base_dir
            .join(place_id.to_string())
            .join(stored_file_name.clone());
        stored.push(StoredImage {
            id: image_id,
            file_name: stored_file_name,
            path,
        });
    }

    Ok(stored)
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
