
pub struct Credentials {
    pub use_authentication: bool,
    pub username: String,
    pub password: String,
}

impl<'a> Credentials {
    pub fn new(username: &String, password: &String) -> Self {
        Credentials {
            use_authentication: true,
            username: username.clone(),
            password: password.clone(),
        }
    }
}

impl Default for Credentials {
    fn default() -> Self {
        Credentials {
            use_authentication: false,
            username: String::new(),
            password: String::new()
        }
    }
}