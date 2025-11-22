use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::auth_service::AuthService;
use crate::jwt::JwtManager;
use crate::repository::auth::AuthRepository;
use crate::repository::place::PlaceRepository;

#[derive(Clone)]
pub struct AppState {
    auth_providers: Arc<HashMap<String, AuthService>>,
    jwt_manager: Arc<JwtManager>,
    auth_repository: AuthRepository,
    place_repository: PlaceRepository,
    place_image_dir: Arc<PathBuf>,
}

impl AppState {
    pub fn new(
        auth_providers: HashMap<String, AuthService>,
        jwt_manager: JwtManager,
        auth_repository: AuthRepository,
        place_repository: PlaceRepository,
        place_image_dir: PathBuf,
    ) -> Self {
        Self {
            auth_providers: Arc::new(auth_providers),
            jwt_manager: Arc::new(jwt_manager),
            auth_repository,
            place_repository,
            place_image_dir: Arc::new(place_image_dir),
        }
    }

    pub fn auth_service(&self, provider: &str) -> Option<AuthService> {
        self.auth_providers.get(provider).cloned()
    }

    pub fn jwt_manager(&self) -> JwtManager {
        self.jwt_manager.as_ref().clone()
    }

    pub fn auth_repository(&self) -> AuthRepository {
        self.auth_repository.clone()
    }

    pub fn place_repository(&self) -> PlaceRepository {
        self.place_repository.clone()
    }

    pub fn place_image_dir(&self) -> Arc<PathBuf> {
        self.place_image_dir.clone()
    }
}
