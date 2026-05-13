use color_eyre::Result;
use tracing::{debug, error};

use crate::{
    App,
    matrix::event::MatrixAction,
    ui::{
        action::{Action, FocusOpts},
        context::{ContextKey, StackEntry},
        context_manager::{cycle_main_screen_backward, cycle_main_screen_forward},
        widgets::status_line::Status,
    },
};

impl App {
    #[allow(clippy::too_many_lines)]
    pub async fn execute_action(&mut self, action: Action) -> Result<()> {
        match action {
            // --- Application ---
            Action::Quit => {
                self.running = false;
            }
            Action::Logout => {
                self.matrix_tx.send(MatrixAction::Logout).await?;
                self.ui
                    .status_line
                    .set_status(Status::Info("Logging out...".to_string()), None);
                self.ui.ctx_mgr.registry.room_list.clear();
                self.ui.ctx_mgr.registry.message_list.clear();
                self.ui.ctx_mgr.enter_login();
            }

            // --- Navigation ---
            Action::PopContext => {
                self.ui.ctx_mgr.pop();
            }
            Action::PushStack(entry, opts) => {
                self.ui.ctx_mgr.push(entry, opts);
            }

            // --- Panel focus cycling ---
            Action::CycleFocusForward => {
                if let Some(current) = self.ui.ctx_mgr.focused_ctx_key() {
                    let next = cycle_main_screen_forward(current);
                    self.ui.ctx_mgr.activate_main_screen(next);
                }
            }
            Action::CycleFocusBackward => {
                if let Some(current) = self.ui.ctx_mgr.focused_ctx_key() {
                    let prev = cycle_main_screen_backward(current);
                    self.ui.ctx_mgr.activate_main_screen(prev);
                }
            }
            Action::FocusSidebar => {
                self.ui.ctx_mgr.activate_main_screen(ContextKey::RoomList);
            }
            Action::FocusMessageList => {
                self.ui
                    .ctx_mgr
                    .activate_main_screen(ContextKey::MessageList);
            }
            Action::FocusMessageInput => {
                self.ui
                    .ctx_mgr
                    .activate_main_screen(ContextKey::MessageInput);
            }

            // --- Messaging ---
            Action::SendMessage => {
                let text = self.ui.ctx_mgr.registry.message_input.take_buffer();
                if !text.trim().is_empty() {
                    let room_id = self.ui.ctx_mgr.registry.room_list.get_selected_room_id();
                    if let Some(room_id) = room_id {
                        self.matrix_tx
                            .send(MatrixAction::SendMessage {
                                room_id,
                                message_body: text,
                            })
                            .await?;
                    } else {
                        error!("SendMessage: no room selected");
                    }
                }
            }
            Action::SelectMessage(idx) => {
                self.ui.ctx_mgr.registry.message_actions.selected_message = Some(idx);
                self.ui.ctx_mgr.push(
                    StackEntry::Overlay(ContextKey::MessageActions),
                    FocusOpts {
                        selected_message: Some(idx),
                        ..Default::default()
                    },
                );
            }
            Action::ReplyToMessage(idx) => {
                self.ui.ctx_mgr.registry.message_input.reply_to = Some(idx);
                self.ui.ctx_mgr.pop();
                self.ui
                    .ctx_mgr
                    .activate_main_screen(ContextKey::MessageInput);
            }
            Action::EditMessage(idx) => {
                // Note: editing by index only; full edit support requires message IDs
                self.ui.ctx_mgr.registry.message_input.editing = Some(idx);
                self.ui.ctx_mgr.pop();
                self.ui
                    .ctx_mgr
                    .activate_main_screen(ContextKey::MessageInput);
            }
            Action::DeleteMessage(idx) => {
                self.ui.ctx_mgr.push(
                    StackEntry::Overlay(ContextKey::ConfirmDelete),
                    FocusOpts {
                        selected_message: Some(idx),
                        ..Default::default()
                    },
                );
            }
            Action::ConfirmDeleteMessage(_idx) => {
                // Full delete support requires message IDs; pop overlays for now
                self.ui.ctx_mgr.pop(); // close ConfirmDelete
                self.ui.ctx_mgr.pop(); // close MessageActions
            }
            Action::ScrollMessagesUp => {
                todo!()
            }
            Action::ScrollMessagesDown => {
                todo!()
            }

            // --- Rooms ---
            Action::SelectRoom(id) => {
                self.ui.ctx_mgr.registry.room_list.select_room(&id);
                self.ui.ctx_mgr.registry.message_list.set_active_room(&id);
                self.ui
                    .ctx_mgr
                    .activate_main_screen(ContextKey::MessageInput);
                // Request messages for the newly selected room
                self.matrix_tx
                    .send(MatrixAction::GetRoomMessages(id))
                    .await?;
            }
            Action::OpenCreateRoom => {
                self.ui.ctx_mgr.push(
                    StackEntry::Overlay(ContextKey::CreateRoom),
                    FocusOpts::default(),
                );
            }
            Action::ConfirmCreateRoom => {
                let name = self.ui.ctx_mgr.registry.create_room.take_name_buffer();
                if !name.trim().is_empty() {
                    // Placeholder: room creation via matrix SDK not yet implemented
                    debug!("Create room: {name}");
                }
                self.ui.ctx_mgr.pop();
            }
            Action::ScrollRoomsUp => {
                self.ui.ctx_mgr.registry.room_list.scroll_up();
            }
            Action::ScrollRoomsDown => {
                self.ui.ctx_mgr.registry.room_list.scroll_down();
            }

            // --- Auth ---
            Action::SubmitLogin => {
                let choice = self.ui.ctx_mgr.registry.login.selected_login_choice();
                let credentials = self.ui.ctx_mgr.registry.login.take_credentials();
                if let Some(choice) = choice {
                    self.matrix_tx
                        .send(MatrixAction::SelectLogin {
                            choice,
                            credentials,
                        })
                        .await?;
                }
            }

            // --- Recovery ---
            Action::ProvideRecoveryKey(key) => {
                self.matrix_tx
                    .send(MatrixAction::ProvideRecoveryKey(key))
                    .await?;
            }
            Action::ConfirmRecoveryKeySaved => {
                self.matrix_tx
                    .send(MatrixAction::ConfirmRecoveryKeySaved)
                    .await?;
            }
        }

        Ok(())
    }
}
