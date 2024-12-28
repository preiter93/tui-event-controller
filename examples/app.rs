use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::WidgetRef;
use std::error::Error;
use std::thread;
use std::time::{Duration, Instant};
use tui_event_controller::EventfulWidget;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

type EventContext<'a> = tui_event_controller::EventContext<'a, AppState, AppEvent>;
type EventController = tui_event_controller::EventController<AppState, AppEvent>;
type IWidget<W> = tui_event_controller::InteractiveWidget<AppState, AppEvent, W>;

#[derive(Debug, Clone)]
enum AppEvent {
    Tick,
}

struct AppState {
    counter: usize,
}

impl AppState {
    fn new() -> Self {
        Self { counter: 0 }
    }
}

enum Page {
    HomePage(IWidget<HomePage>),
}

fn main() -> Result<()> {
    let controller = EventController::new();
    spawn_event_publisher(&controller, 500);

    let mut state = AppState::new();
    let app = IWidget::new(App::new(&controller), &controller);

    let start_instant = Instant::now();
    while start_instant.elapsed() < Duration::from_secs(2) {
        let area = Rect::default();
        let mut buf = Buffer::empty(area);

        app.render_ref(Rect::default(), &mut buf);
        controller.recv_and_notify(&mut state)?;
    }

    Ok(())
}

struct App {
    active_page: Page,
}

impl App {
    fn new(controller: &EventController) -> Self {
        let home_page = IWidget::new(HomePage::default(), &controller);
        Self {
            active_page: Page::HomePage(home_page),
        }
    }
}

impl EventfulWidget<AppState, AppEvent> for App {
    fn unique_key() -> String {
        String::from("App")
    }

    fn on_event(ctx: EventContext, _: &mut AppState, _: Option<Rect>) {
        match ctx.event {
            AppEvent::Tick => {
                println!("App: tick");
            }
        }
    }
}

impl WidgetRef for App {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        match &self.active_page {
            Page::HomePage(widget) => widget.render_ref(area, buf),
        }
    }
}

#[derive(Default)]
struct HomePage;

impl EventfulWidget<AppState, AppEvent> for HomePage {
    fn unique_key() -> String {
        String::from("HomePage")
    }

    fn on_event(ctx: EventContext, state: &mut AppState, _: Option<Rect>) {
        match ctx.event {
            AppEvent::Tick => {
                state.counter += 1;
                println!("HomePage: tick {:}", state.counter);
            }
        }
    }
}

impl WidgetRef for HomePage {
    fn render_ref(&self, _area: Rect, _buf: &mut Buffer) {}
}

fn spawn_event_publisher(controller: &EventController, tick_rate_ms: u64) {
    let tick_rate = Duration::from_millis(tick_rate_ms);

    let sender = controller.get_sender();

    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // Emit tick events
            if last_tick.elapsed() >= tick_rate {
                let _ = sender.send(AppEvent::Tick);
                last_tick = Instant::now();
            }

            // Spawn other events...
        }
    });
}
