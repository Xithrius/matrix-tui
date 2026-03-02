#[derive(Clone, Debug)]
pub enum LoginCredentials {
    Password {
        username: String,
        password: String,
    },
    #[allow(dead_code)]
    Other,
}
