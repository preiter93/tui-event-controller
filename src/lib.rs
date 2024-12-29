//! # Description
//!
//! A event controller for [ratatui](https://github.com/ratatui/ratatui) apps.
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
//! struct App;
//!
//! impl WidgetRef for App {
//!     fn render_ref(&self, _area: Rect, _buf: &mut Buffer) {}
//! }
//!
//! impl EventfulWidget<AppState, AppEvent> for App {
//!     fn unique_key() -> String {
//!         String::from("App")
//!     }
//!
//!     fn on_event(ctx: EventContext, state: &mut AppState, _: Option<Rect>) {
//!         match ctx.event {
//!             AppEvent::Tick => {
//!                 state.counter += 1;
//!                 println!("App: tick {:}", state.counter);
//!             }
//!             AppEvent::Key(_) => {}
//!         }
//!     }
//! }
//!
//! type Result<T> = std::result::Result<T, Box<dyn Error>>;
//!
//! fn main() -> Result<()> {
//!     // Initialize the event controller.
//!     let controller = EventController::new();
//!
//!     // Start an event loop that processes events.
//!     spawn_event_loop(&controller);
//!
//!     // Initialize the application state.
//!     let mut state = AppState::default();
//!
//!     // Create the interactive application widget.
//!     // The `IWidget` constructor registers the `on_event` callback.
//!     let app = IWidget::new(App, &controller);
//!
//!     while !state.should_quit {
//!         // Draw the app...
//!
//!         // Wait for events and notify all event listeners.
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
//!
//! # Apps build with `tui-event-controller`
//!
//! - [Mantui](https://github.com/preiter93/mantui)
mod controller;
mod widget;

pub use controller::{EventContext, EventController};
pub use widget::{EventfulWidget, InteractiveStatefulWidget, InteractiveWidget};
