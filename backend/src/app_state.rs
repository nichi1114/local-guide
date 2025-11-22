use std::collections::HashMap;
use std::sync::Arc;

use crate::auth_service::AuthService;
use crate::jwt::JwtManager;
use crate::repository::auth::AuthRepository;
use crate::repository::image_store::ImageStore;
use crate::repository::place::PlaceRepository;

#[derive(Clone)]
pub struct AppState {
    auth_providers: Arc<HashMap<String, AuthService>>,
    jwt_manager: Arc<JwtManager>,
    auth_repository: AuthRepository,
    place_repository: PlaceRepository,
    image_store: ImageStore,
}

impl AppState {
    pub fn new(
        auth_providers: HashMap<String, AuthService>,
        jwt_manager: JwtManager,
        auth_repository: AuthRepository,
        place_repository: PlaceRepository,
        image_store: ImageStore,
    ) -> Self {
        Self {
            auth_providers: Arc::new(auth_providers),
            jwt_manager: Arc::new(jwt_manager),
            auth_repository,
            place_repository,
            image_store,
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

    pub fn image_store(&self) -> ImageStore {
        self.image_store.clone()
    }
}
