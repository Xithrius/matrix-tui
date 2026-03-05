#[derive(Clone, Debug)]
pub struct MatrixMessage {
    pub name: String,
    pub content: String,
}

impl MatrixMessage {
    pub const fn new(name: String, content: String) -> Self {
        Self { name, content }
    }
}
