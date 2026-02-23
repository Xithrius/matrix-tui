use tui::{Frame, crossterm::event::KeyEvent, layout::Rect};

/// `Component` is a trait that represents a visual and interactive element of the user interface.
///
/// Implementors of this trait can be registered with the main application loop and will be able to
/// receive events, update state, and be rendered on the screen.
pub trait Component {
    /// Handle key events.
    fn handle_key_event(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        let _ = key;
        Ok(())
    }

    /// Render the component on the screen.
    fn draw(&mut self, frame: &mut Frame, area: Rect);
}
