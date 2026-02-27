use std::convert::Into;

use color_eyre::Result;
use rustyline::{
    At, Word,
    line_buffer::{self, ChangeListener, DeleteListener, LineBuffer},
};
use tui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    prelude::*,
    widgets::{Block, Paragraph},
};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::ui::component::Component;

const LINE_BUFFER_CAPACITY: usize = 1024;

/// Acquiring the horizontal position of the cursor so it can be rendered visually.
pub fn get_cursor_position(line_buffer: &LineBuffer) -> usize {
    line_buffer
        .as_str()
        .grapheme_indices(true)
        .take_while(|(offset, _)| *offset != line_buffer.pos())
        .map(|(_, cluster)| cluster.width())
        .sum()
}

#[derive(Debug)]
pub struct InputListener;

impl ChangeListener for InputListener {
    fn insert_char(&mut self, _idx: usize, _c: char) {}
    fn insert_str(&mut self, _idx: usize, _string: &str) {}
    fn replace(&mut self, _idx: usize, _old: &str, _new: &str) {}
}

impl DeleteListener for InputListener {
    fn delete(&mut self, _idx: usize, _string: &str, _dir: line_buffer::Direction) {}
}

pub struct UserInputWidget {
    title: Option<String>,
    focused: bool,

    input_listener: InputListener,
    input: LineBuffer,
    /// The input changed on the last keystroke.
    input_changed: bool,
}

impl UserInputWidget {
    pub fn new(title: Option<impl Into<String>>) -> Self {
        Self {
            title: title.map(Into::into),
            focused: false,

            input_listener: InputListener,
            input: LineBuffer::with_capacity(LINE_BUFFER_CAPACITY),
            input_changed: false,
        }
    }

    pub const fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub const fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn set_input(&mut self, input: &str) {
        self.input
            .update(input, input.len(), &mut self.input_listener);
    }

    pub fn clear(&mut self) {
        self.set_input("");
    }

    pub fn get_input(&self) -> &str {
        self.input.as_str()
    }

    #[allow(dead_code)]
    pub const fn input_changed(&self) -> bool {
        self.input_changed
    }
}

impl Component for UserInputWidget {
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        if !self.is_focused() {
            return Ok(());
        }

        let previous_input = self.input.to_string();

        match (
            key.code,
            key.modifiers.contains(KeyModifiers::CONTROL),
            key.modifiers.contains(KeyModifiers::ALT),
        ) {
            (KeyCode::Right, false, _) | (KeyCode::Char('f'), true, false) => {
                if self.input.next_pos(1).is_none() {
                    self.input.move_end();
                } else {
                    self.input.move_forward(1);
                }
            }
            (KeyCode::Left, false, _) | (KeyCode::Char('b'), true, false) => {
                self.input.move_backward(1);
            }
            (KeyCode::Char('a'), true, false) => {
                self.input.move_home();
            }
            (KeyCode::Char('e'), true, false) => {
                self.input.move_end();
            }
            (KeyCode::Char('f'), false, true) | (KeyCode::Right, true, false) => {
                self.input.move_to_next_word(At::AfterEnd, Word::Emacs, 1);
            }
            (KeyCode::Char('b'), false, true) | (KeyCode::Left, true, false) => {
                self.input.move_to_prev_word(Word::Emacs, 1);
            }
            (KeyCode::Char('t'), true, false) => {
                self.input.transpose_chars(&mut self.input_listener);
            }
            (KeyCode::Char('t'), false, true) => {
                self.input.transpose_words(1, &mut self.input_listener);
            }
            (KeyCode::Char('u'), true, false) => {
                self.input.discard_line(&mut self.input_listener);
            }
            (KeyCode::Char('k'), true, false) => {
                self.input.kill_line(&mut self.input_listener);
            }
            (KeyCode::Char('w'), true, false) => {
                self.input
                    .delete_prev_word(Word::Emacs, 1, &mut self.input_listener);
            }
            (KeyCode::Delete, _, _) | (KeyCode::Char('d'), true, false) => {
                self.input.delete(1, &mut self.input_listener);
            }
            (KeyCode::Backspace, _, _) => {
                self.input.backspace(1, &mut self.input_listener);
            }
            (KeyCode::Char(c), false, false) => {
                self.input.insert(c, 1, &mut self.input_listener);
            }
            _ => {}
        }

        let current_input = self.input.as_str();
        self.input_changed = previous_input != current_input;

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let input = Paragraph::new(self.input.as_str())
            .block(Block::bordered().title(self.title.clone().unwrap_or_default()));
        frame.render_widget(input, area);

        if !self.is_focused() {
            return;
        }

        let cursor_pos = get_cursor_position(&self.input);
        let cursor_frame_pos = Position::new(
            (area.x + cursor_pos as u16 + 1).min(area.x + area.width.saturating_sub(2)),
            area.y + 1,
        );
        frame.set_cursor_position(cursor_frame_pos);
    }
}
