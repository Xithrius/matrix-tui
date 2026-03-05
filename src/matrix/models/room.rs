use matrix_sdk::Room;

#[derive(Debug, Clone)]
pub struct MatrixRoom {
    pub id: String,
    pub name: Option<String>,
}

impl MatrixRoom {
    pub const fn new(id: String, name: Option<String>) -> Self {
        Self { id, name }
    }
}

impl From<Room> for MatrixRoom {
    fn from(value: Room) -> Self {
        Self {
            id: value.room_id().as_str().to_owned(),
            name: value.name(),
        }
    }
}
