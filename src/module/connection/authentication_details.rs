
pub struct AuthenticationDetails {
    pub use_authentication: bool,
    pub username: String,
    pub password: String,
}

impl AuthenticationDetails {
    pub fn new(username: Option<String>, password: Option<String>) -> Self {
        AuthenticationDetails {
            use_authentication: true,
            username: username.unwrap_or_default(),
            password: password.unwrap_or_default(),
        }
    }
}

impl Default for AuthenticationDetails {
    fn default() -> Self {
        AuthenticationDetails {
            use_authentication: false,
            username: String::new(),
            password: String::new()
        }
    }
}