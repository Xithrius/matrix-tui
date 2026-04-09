use chrono::{DateTime, Local};

#[derive(Clone, Debug)]
pub struct MatrixMessage {
    pub datetime: DateTime<Local>,
    pub name: String,
    pub content: String,
}

impl MatrixMessage {
    pub const fn new(datetime: DateTime<Local>, name: String, content: String) -> Self {
        Self {
            datetime,
            name,
            content,
        }
    }
}
