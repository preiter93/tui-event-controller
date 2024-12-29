//! # Description
//!
//! A event controller for ratatui apps.
//!
//! # Example
//!
//! ```no_run
//! use tui_event_controller::EventfulWidget;
//! use ratatui::prelude::*;
//! use ratatui::widgets::WidgetRef;
//! use ratatui::crossterm::event::{self, KeyEvent, Event as CrosstermEvent};
//! use std::{thread, error::Error, time};
//!
//! type EventContext<'a> = tui_event_controller::EventContext<'a, AppState, AppEvent>;
//! type EventController = tui_event_controller::EventController<AppState, AppEvent>;
//! type IWidget<W> = tui_event_controller::InteractiveWidget<AppState, AppEvent, W>;
//!
//! enum AppEvent {
//!     Tick,
//!     Key(KeyEvent),
//! }
//!
//! #[derive(Default)]
//! struct AppState {
//!     counter: usize,
//!     should_quit: bool,
//! }
//!
//! #[derive(Default)]
//! struct HomePage;
//!
//! impl WidgetRef for HomePage {
//!     fn render_ref(&self, _area: Rect, _buf: &mut Buffer) {}
//! }
//!
//! impl EventfulWidget<AppState, AppEvent> for HomePage {
//!     fn unique_key() -> String {
//!         String::from("HomePage")
//!     }
//!
//!     fn on_event(ctx: EventContext, state: &mut AppState, _: Option<Rect>) {
//!         match ctx.event {
//!             AppEvent::Tick => {
//!                 state.counter += 1;
//!                 println!("HomePage: tick {:}", state.counter);
//!             }
//!             AppEvent::Key(_) => {}
//!         }
//!     }
//! }
//!
//! type Result<T> = std::result::Result<T, Box<dyn Error>>;
//!
//! fn main() -> Result<()> {
//!     let controller = EventController::new();
//!     spawn_event_loop(&controller);
//!
//!     let mut state = AppState::default();
//!
//!     let start_instant = time::Instant::now();
//!     while !state.should_quit {
//!         // Render your app
//!         // app.draw()
//!
//!         // Handle your events
//!         controller.recv_and_notify(&mut state)?;
//!     }
//!
//!     Ok(())
//! }
//!
//! fn spawn_event_loop(controller: &EventController) {
//!     let tick_rate = time::Duration::from_millis(500);
//!
//!     let sender = controller.get_sender();
//!     thread::spawn(move || {
//!         let mut last_tick = time::Instant::now();
//!         loop {
//!             // Emit crossterm events
//!             let timeout = tick_rate.saturating_sub(last_tick.elapsed());
//!             if event::poll(timeout).expect("unable to poll events") {
//!                 match event::read().expect("unable to read event") {
//!                     CrosstermEvent::Key(event) => {
//!                         let _ = sender.send(AppEvent::Key(event));
//!                     }
//!                     _ => {}
//!                 }
//!             }
//!
//!             // Emit tick events
//!             if last_tick.elapsed() >= tick_rate {
//!                 let _ = sender.send(AppEvent::Tick);
//!                 last_tick = time::Instant::now();
//!             }
//!         }
//!     });
//! }
//! ```

mod controller;
mod widget;

pub use controller::{EventContext, EventController};
pub use widget::{EventfulWidget, InteractiveStatefulWidget, InteractiveWidget};
