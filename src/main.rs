use crossterm_events::{EventController, EventWidget, EventfulWidget};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::layout::{Position, Rect};
use ratatui::widgets::Widget;
use std::error::Error;
use std::thread;
use std::time::{Duration, Instant};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// Global app state.
struct AppState {
    counter: usize,
}

impl AppState {
    fn new() -> Self {
        Self { counter: 0 }
    }
}

/// App events.
#[derive(Debug, Clone)]
enum AppEvent {
    /// Terminal tick.
    Tick,

    /// Mouse event.
    MouseEvent(MouseEvent),
}

/// Emits regular tick events in a separate thread.
fn emit_events(ctrl: &EventController<AppState, AppEvent>, tick_rate_ms: u64) {
    let tick_rate = Duration::from_millis(tick_rate_ms);

    let sender = ctrl.sender.clone();

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            if last_tick.elapsed() >= tick_rate {
                // Emit Mouse Events
                let event = MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column: 0,
                    row: 0,
                    modifiers: KeyModifiers::NONE,
                };
                let _ = sender.send(AppEvent::MouseEvent(event));

                // Emit tick events
                let _ = sender.send(AppEvent::Tick);
                last_tick = Instant::now();
            }
        }
    });
}

enum Page {
    MyWidget(EventfulWidget<AppState, AppEvent, MyWidget>),
}

/// The app.
struct App {
    /// The currently active page.
    active_page: Page,
}

impl App {
    /// Creates a new instance of `App`.
    ///
    /// # Arguments
    /// - `active_page`: The initially active page.
    fn new(active_page: Page) -> Self {
        Self { active_page }
    }
}

impl Widget for &mut App {
    /// Renders the app.
    fn render(self, area: Rect, buf: &mut Buffer) {
        match &mut self.active_page {
            Page::MyWidget(widget) => widget.render(area, buf),
        }
    }
}

fn main() -> Result<()> {
    let event_ctrl = EventController::new();
    let mut state = AppState::new();
    emit_events(&event_ctrl.borrow(), 500);

    let widget = EventfulWidget::new(MyWidget::new(), &event_ctrl);
    let mut app = App::new(Page::MyWidget(widget));

    let start_instant = Instant::now();

    let area = Rect::new(0, 0, 1, 1);
    let mut buf = Buffer::empty(area);

    while start_instant.elapsed() < Duration::from_secs(2) {
        event_ctrl.borrow_mut().handle_events(&mut state)?;
        app.render(area, &mut buf);
    }

    Ok(())
}

struct MyWidget {}

impl Widget for &MyWidget {
    fn render(self, _area: Rect, _buf: &mut Buffer) {}
}

impl MyWidget {
    fn new() -> Self {
        Self {}
    }
}

impl EventWidget<AppState, AppEvent> for MyWidget {
    fn key() -> String {
        String::from("MyWidget")
    }

    fn handle_events(
        ctrl: &mut EventController<AppState, AppEvent>,
        state: &mut AppState,
        event: &AppEvent,
        area: Option<Rect>,
    ) {
        match event {
            AppEvent::Tick => {
                state.counter += 1;
                log_to_file(format!("tick {:}", state.counter));
            }
            AppEvent::MouseEvent(e) => {
                let pos = Position::new(e.column, e.row);
                if area.filter(|a| a.contains(pos)).is_some() {
                    log_to_file("click");
                }

                ctrl.add_listener("overwrite", |_, _, _| {
                    log_to_file("inner event");
                });
            }
        }
    }
}

#[allow(dead_code)]
fn log_to_file<T: std::fmt::Debug>(data: T) {
    use std::fs::OpenOptions;
    use std::io::Write;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug.log")
        .unwrap();

    if let Err(e) = writeln!(file, "{data:?}") {
        eprintln!("Couldn't write to file: {e}");
    }
}
