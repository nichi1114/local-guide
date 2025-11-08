use std::collections::HashMap;
use std::sync::Arc;

use crate::auth_service::AuthService;

#[derive(Clone)]
pub struct AppState {
    providers: Arc<HashMap<String, AuthService>>,
}

impl AppState {
    pub fn new(providers: HashMap<String, AuthService>) -> Self {
        Self {
            providers: Arc::new(providers),
        }
    }

    pub fn auth_service(&self, provider: &str) -> Option<AuthService> {
        self.providers.get(provider).cloned()
    }
}
