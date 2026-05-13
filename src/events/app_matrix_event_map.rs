use color_eyre::Result;

use crate::{
    App,
    matrix::event::{MatrixAction, MatrixEvent},
    ui::widgets::status_line::Status,
};
impl App {
    #[allow(clippy::too_many_lines)]
    pub async fn handle_matrix_event(&mut self, event: MatrixEvent) -> Result<()> {
        match event {
            MatrixEvent::RestoringSession => {
                self.ui
                    .status_line
                    .set_status(Status::Info("Restoring session...".to_string()), None);
                self.ui.header.set_loading(true);
                self.ui.header.set_mode("Restoring session".to_string());
            }
            MatrixEvent::SuccessfulSessionRestore => {
                self.ui.status_line.set_status(
                    Status::Info("Session restored, setting up encryption...".to_string()),
                    None,
                );
            }
            MatrixEvent::LoginChoices(choices) => {
                self.ui.ctx_mgr.registry.login.set_login_choices(choices);
                self.ui
                    .status_line
                    .set_status(Status::Info("Select login option".to_string()), None);
            }
            MatrixEvent::LoggingIn => {
                self.ui
                    .status_line
                    .set_status(Status::Info("Logging in...".to_string()), None);
            }
            MatrixEvent::SuccessfulLogin => {
                self.ui.status_line.set_status(
                    Status::Info("Login successful, setting up encryption...".to_string()),
                    None,
                );
            }
            MatrixEvent::LoginFailed => {
                self.ui.ctx_mgr.enter_login();
                self.ui
                    .status_line
                    .set_status(Status::Error("Login failed".to_string()), Some(5));
            }
            MatrixEvent::NeedsRecoveryKey => {
                self.ui.ctx_mgr.enter_recovery();
                self.ui.status_line.set_status(
                    Status::Info("Enter your recovery key to restore encryption".to_string()),
                    None,
                );
            }
            MatrixEvent::ShowNewRecoveryKey(key) => {
                self.ui.ctx_mgr.registry.recovery.show_new_key(key);
                // ctx_mgr is already in Recovery fullscreen; just update the widget.
            }
            MatrixEvent::EncryptionSetupComplete => {
                self.ui.header.set_loading(false);
                self.ui.ctx_mgr.enter_session();
                self.ui
                    .status_line
                    .set_status(Status::Info("Encryption configured".to_string()), Some(5));
            }
            MatrixEvent::KnownRooms(rooms) => {
                let first_room = rooms.first().map(|r| r.id.clone());

                for room in rooms {
                    let room_id = room.id.clone();
                    self.ui.ctx_mgr.registry.room_list.push_room(room);
                    self.matrix_tx
                        .send(MatrixAction::GetRoomMessages(room_id))
                        .await?;
                }

                if let Some(id) = first_room {
                    self.ui.ctx_mgr.registry.room_list.select_room(&id);
                    self.ui.ctx_mgr.registry.message_list.set_active_room(&id);
                }
            }
            MatrixEvent::RoomMessages {
                room_id,
                mut messages,
            } => {
                messages.sort_by_key(|m| m.datetime);
                for message in messages {
                    self.ui
                        .ctx_mgr
                        .registry
                        .message_list
                        .push_message(&room_id, message);
                }
            }
            MatrixEvent::Message { room_id, message } => {
                self.ui
                    .ctx_mgr
                    .registry
                    .message_list
                    .push_message(&room_id, message);
            }
        }

        Ok(())
    }
}
