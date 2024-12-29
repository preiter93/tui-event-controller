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

#[derive(Default)]
struct AppState {
    counter: usize,
    should_quit: bool,
}

enum ActivePage {
    HomePage(IWidget<HomePage>),
}

struct App {
    active_page: ActivePage,
}

impl App {
    fn new(controller: &EventController) -> Self {
        let home_page = IWidget::new(HomePage::default(), &controller);
        Self {
            active_page: ActivePage::HomePage(home_page),
        }
    }
}

impl EventfulWidget<AppState, AppEvent> for App {
    fn unique_key() -> String {
        String::from("App")
    }

    fn on_event(ctx: EventContext, state: &mut AppState, _: Option<Rect>) {
        match ctx.event {
            AppEvent::Tick => {
                println!("App: tick");
                if state.counter == 3 {
                    state.should_quit = true;
                }
            }
        }
    }
}

impl WidgetRef for App {
    fn render_ref(&self, area: Rect, buf: &mut Buffer) {
        match &self.active_page {
            ActivePage::HomePage(widget) => widget.render_ref(area, buf),
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

fn main() -> Result<()> {
    // Initialize the event controller.
    let controller = EventController::new();

    // Start an event loop that processes events.
    spawn_event_loop(&controller);

    // Initialize the application state.
    let mut state = AppState::default();

    // Create the application widget.
    // The `IWidget` constructor registers the `EventfulWiget::on_event` event-listener.
    let _app = IWidget::new(App::new(&controller), &controller);

    while !state.should_quit {
        // Draw the app...

        // Wait for events and notify all event listeners.
        controller.recv_and_notify(&mut state)?;
    }

    Ok(())
}

fn spawn_event_loop(controller: &EventController) {
    let tick_rate = Duration::from_millis(500);

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
