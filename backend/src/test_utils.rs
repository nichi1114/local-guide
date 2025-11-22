#[cfg(test)]
pub mod router {
    use crate::app_state::AppState;
    use crate::db::create_pool;
    use crate::jwt::JwtManager;
    use crate::repository::auth::{AuthRepository, UserRecord};
    use crate::repository::image_store::ImageStore;
    use crate::repository::place::PlaceRepository;
    use crate::sql_init::run_initialization;
    use axum::body::Body;
    use axum::http::Request;
    use axum::response::Response;
    use axum::Router;
    use http_body_util::BodyExt;
    use serde::de::DeserializeOwned;
    use sqlx::PgPool;
    use std::collections::HashMap;
    use std::path::Path;
    use tempfile::TempDir;
    use tower::ServiceExt;
    use uuid::Uuid;

    pub const TEST_JWT_SECRET: &str = "secret";

    pub struct TestContext {
        pub app: Router,
        pub pool: PgPool,
        pub jwt: JwtManager,
        temp_dir: TempDir,
        auth_repo: AuthRepository,
        place_repo: PlaceRepository,
    }

    impl TestContext {
        pub async fn new(build_router: impl FnOnce(AppState) -> Router) -> Self {
            let pool = setup_pool().await;
            let temp_dir = TempDir::new().expect("temp dir");
            let auth_repo = AuthRepository::new(pool.clone());
            let place_repo = PlaceRepository::new(pool.clone());
            let image_store = ImageStore::new(temp_dir.path().to_path_buf()).expect("image store");
            let jwt = JwtManager::new(TEST_JWT_SECRET.to_string(), 3600);

            let providers = HashMap::new();
            let state = AppState::new(
                providers,
                jwt.clone(),
                auth_repo.clone(),
                place_repo.clone(),
                image_store,
            );

            let app = build_router(state);

            Self {
                app,
                pool,
                jwt,
                temp_dir,
                auth_repo,
                place_repo,
            }
        }

        pub fn image_dir(&self) -> &Path {
            self.temp_dir.path()
        }

        pub fn auth_repo(&self) -> AuthRepository {
            self.auth_repo.clone()
        }

        pub fn place_repo(&self) -> PlaceRepository {
            self.place_repo.clone()
        }

        pub async fn insert_user(&self) -> UserRecord {
            let id = Uuid::new_v4();
            let email = format!("user-{}@example.com", id);
            let name = format!("User {}", id);
            sqlx::query("INSERT INTO users (id, email, name) VALUES ($1, $2, $3)")
                .bind(id)
                .bind(&email)
                .bind(&name)
                .execute(&self.pool)
                .await
                .expect("insert user");

            UserRecord {
                id,
                email: Some(email),
                name: Some(name),
                avatar_url: None,
            }
        }

        pub async fn send_request(&self, request: Request<Body>) -> Result<Response, axum::Error> {
            self.app
                .clone()
                .oneshot(request)
                .await
                .map_err(axum::Error::new)
        }
    }

    pub enum Part {
        Text {
            name: &'static str,
            value: String,
        },
        File {
            name: &'static str,
            filename: &'static str,
            content_type: &'static str,
            data: Vec<u8>,
        },
    }

    impl Part {
        pub fn text(name: &'static str, value: impl Into<String>) -> Self {
            Self::Text {
                name,
                value: value.into(),
            }
        }

        pub fn file(
            name: &'static str,
            filename: &'static str,
            content_type: &'static str,
            data: Vec<u8>,
        ) -> Self {
            Self::File {
                name,
                filename,
                content_type,
                data,
            }
        }
    }

    pub fn multipart_body(parts: Vec<Part>) -> (String, Vec<u8>) {
        let boundary = format!("boundary-{}", Uuid::new_v4());
        let mut body = Vec::new();
        for part in parts {
            body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
            match part {
                Part::Text { name, value } => {
                    body.extend_from_slice(
                        format!("Content-Disposition: form-data; name=\"{}\"\r\n\r\n", name)
                            .as_bytes(),
                    );
                    body.extend_from_slice(value.as_bytes());
                    body.extend_from_slice(b"\r\n");
                }
                Part::File {
                    name,
                    filename,
                    content_type,
                    data,
                } => {
                    body.extend_from_slice(
                        format!(
                            "Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r\n",
                            name, filename
                        )
                        .as_bytes(),
                    );
                    body.extend_from_slice(
                        format!("Content-Type: {}\r\n\r\n", content_type).as_bytes(),
                    );
                    body.extend_from_slice(&data);
                    body.extend_from_slice(b"\r\n");
                }
            }
        }
        body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());
        (boundary, body)
    }

    pub async fn parse_json<T: DeserializeOwned>(response: Response) -> T {
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        serde_json::from_slice(&bytes).expect("json response")
    }

    async fn setup_pool() -> PgPool {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .or_else(|_| std::env::var("DATABASE_URL"))
            .unwrap_or_else(|_| {
                "postgres://postgres:postgres@localhost:5432/local_guide_test".into()
            });

        let pool = create_pool(&database_url)
            .await
            .expect("connect to postgres");
        run_initialization(&pool).await.expect("apply schema");

        sqlx::query(
            "TRUNCATE TABLE place_images, places, oauth_identities, users RESTART IDENTITY",
        )
        .execute(&pool)
        .await
        .expect("truncate tables");

        pool
    }
}
