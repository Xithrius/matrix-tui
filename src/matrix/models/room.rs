use matrix_sdk::Room;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixRoom {
    pub id: String,
    pub name: Option<String>,
}

impl From<Room> for MatrixRoom {
    fn from(value: Room) -> Self {
        Self {
            id: value.room_id().as_str().to_owned(),
            name: value.name(),
        }
    }
}
