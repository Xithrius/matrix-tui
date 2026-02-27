use std::time::{Duration, Instant};

use color_eyre::{Result, eyre::WrapErr};
use tokio::sync::mpsc::{Receiver, Sender};
use tui::crossterm::event;

use crate::{config::CoreConfig, events::Event};

#[derive(Debug)]
pub struct EventHandler {
    rx: Receiver<Event>,
}

impl EventHandler {
    pub fn new(config: &CoreConfig, event_tx: Sender<Event>, event_rx: Receiver<Event>) -> Self {
        let fps = config.terminal.frame_rate;
        let actor = EventThread::new(event_tx, fps);
        tokio::spawn(async move { actor.run().await });

        Self { rx: event_rx }
    }

    /// Receives an event from the sender.
    ///
    /// This function blocks until an event is received.
    ///
    /// # Errors
    ///
    /// This function returns an error if the sender channel is disconnected. This can happen if an
    /// error occurs in the event thread. In practice, this should not happen unless there is a
    /// problem with the underlying terminal.
    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}

/// A thread that handles reading crossterm events and emitting tick events on a regular schedule.
struct EventThread {
    fps: f64,
    event_tx: Sender<Event>,
}

impl EventThread {
    const fn new(event_tx: Sender<Event>, fps: f64) -> Self {
        Self { fps, event_tx }
    }

    /// Runs the event thread.
    ///
    /// This function emits tick events at a fixed rate and polls for crossterm events in between.
    async fn run(self) -> Result<()> {
        let tick_interval = Duration::from_secs_f64(1.0 / self.fps);
        let mut last_tick = Instant::now();

        loop {
            // Emit tick events at a fixed rate
            let timeout = tick_interval.saturating_sub(last_tick.elapsed());
            if timeout == Duration::ZERO {
                last_tick = Instant::now();
                self.send(Event::Tick).await;
            }

            // Poll for crossterm events, ensuring that we don't block the tick interval
            if event::poll(timeout).wrap_err("failed to poll for crossterm events")? {
                let event = event::read().wrap_err("failed to read crossterm event")?;
                self.send(Event::Crossterm(event)).await;
            }
        }
    }

    async fn send(&self, event: Event) {
        // Ignores the result because shutting down the app drops the receiver, which causes the send
        // operation to fail. This is expected behavior and should not panic.
        let _ = self.event_tx.send(event).await;
    }
}
