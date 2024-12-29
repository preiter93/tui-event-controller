# tui-event-controller

## Description

A event controller for ratatui apps.

## Example

```rust
use tui_event_controller::EventfulWidget;
use ratatui::prelude::*;
use ratatui::widgets::WidgetRef;

type EventContext<'a> = tui_event_controller::EventContext<'a, AppState, AppEvent>;
type EventController = tui_event_controller::EventController<AppState, AppEvent>;
type IWidget<W> = tui_event_controller::InteractiveWidget<AppState, AppEvent, W>;

enum AppEvent {
    Tick,
}

struct AppState {
    counter: usize,
}

#[derive(Default)]
struct HomePage;

impl WidgetRef for HomePage {
    fn render_ref(&self, _area: Rect, _buf: &mut Buffer) {}
}

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

let controller = EventController::new();
let home_page = IWidget::new(HomePage::default(), &controller);
```

License: MIT
