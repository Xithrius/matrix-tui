#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum LoginCredentials {
    Password { username: String, password: String },
    Other,
}
