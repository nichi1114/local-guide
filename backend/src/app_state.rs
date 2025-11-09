use std::collections::HashMap;
use std::sync::Arc;

use crate::auth_service::AuthService;
use crate::jwt::JwtManager;
use crate::repository::auth::AuthRepository;

#[derive(Clone)]
pub struct AppState {
    auth_providers: Arc<HashMap<String, AuthService>>,
    jwt_manager: Arc<JwtManager>,
    auth_repository: AuthRepository,
}

impl AppState {
    pub fn new(
        auth_providers: HashMap<String, AuthService>,
        jwt_manager: JwtManager,
        auth_repository: AuthRepository,
    ) -> Self {
        Self {
            auth_providers: Arc::new(auth_providers),
            jwt_manager: Arc::new(jwt_manager),
            auth_repository,
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
}
