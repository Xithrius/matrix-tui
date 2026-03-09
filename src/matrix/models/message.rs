#[derive(Clone, Debug)]
pub struct MatrixMessage {
    pub datetime: String,
    pub name: String,
    pub content: String,
}

impl MatrixMessage {
    pub const fn new(datetime: String, name: String, content: String) -> Self {
        Self {
            datetime,
            name,
            content,
        }
    }
}
