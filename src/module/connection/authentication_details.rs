
pub struct AuthenticationDetails {
    pub use_authentication: bool,
    pub username: String,
    pub password: String,
}

impl<'a> AuthenticationDetails {
    pub fn new(username: &String, password: &String) -> Self {
        AuthenticationDetails {
            use_authentication: true,
            username: username.clone(),
            password: password.clone(),
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