use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::rc::Rc;
use std::sync::mpsc::{self};

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

type EventCallback<S, E> = Rc<dyn Fn(&mut S, &E) + 'static>;

/// An event controller for ratatui apps.
///
/// ```
/// use ratatui::prelude::*;
/// use crossterm_events::{EventController, EventfulWidget, EventListener};
///
/// #[derive(Default)]
/// struct AppState {
///     counter: usize,
/// }
///
/// /// App events.
/// #[derive(Debug)]
/// enum AppEvent {
///     Tick,
/// }
///
/// #[derive(Default)]
/// struct MyWidget {}
///
/// impl Widget for &MyWidget {
///     fn render(self, area: Rect, buf: &mut Buffer) {}
/// }
///
/// impl MyWidget {
///     const KEY: &str = "foo";
///
///     fn new() -> Self {
///         Self {}
///     }
/// }
///
/// impl EventListener<AppState, AppEvent> for MyWidget {
///     fn key() -> String {
///         String::from("MyWidget")
///     }
///
///     fn handle_event(state: &mut AppState, event: &AppEvent, area: Option<Rect>) {
///         state.counter += 1;
///         println!("event: {event:?} | counter: {}", state.counter);
///     }
/// }
///
/// let mut state = AppState::default();
/// let mut event_ctrl = EventController::new();
///
/// let widget = MyWidget::default();
/// let widget = EventfulWidget::new(widget, &event_ctrl);
/// ```
pub struct EventController<S, E> {
    /// Event sender channel.
    pub sender: mpsc::Sender<E>,

    /// Event receiver channel.
    receiver: mpsc::Receiver<E>,

    /// Registered callbacks.
    callbacks: HashMap<String, EventCallback<S, E>>,
}

impl<S, E> EventController<S, E> {
    /// Creates a new instance of [`EventHandler`].
    #[must_use]
    pub fn new() -> Rc<RefCell<Self>> {
        let (sender, receiver) = mpsc::channel();
        let callbacks = HashMap::default();

        Rc::new(RefCell::new(Self {
            sender,
            receiver,
            callbacks,
        }))
    }

    /// Adds a new listener.
    ///
    /// # Example
    /// ```
    /// use crossterm_events::EventController;
    ///
    /// struct AppState;
    /// #[derive(Debug)]
    /// struct AppEvent;
    ///
    /// let mut event_ctrl = EventController::<AppState, AppEvent>::new();
    /// event_ctrl.borrow_mut().add_listener("foo", move |_state, event| {
    ///     println!("received: {event:?}");
    /// });
    /// ```
    pub fn add_listener<F>(&mut self, id: &str, callback: F)
    where
        F: Fn(&mut S, &E) + 'static,
    {
        self.callbacks.insert(id.to_string(), Rc::new(callback));
    }

    /// Removes a listener.
    ///
    /// # Example
    /// ```
    /// use crossterm_events::EventController;
    ///
    /// struct AppState;
    /// struct AppEvent;
    ///
    /// let mut event_ctrl = EventController::<AppState, AppEvent>::new();
    /// event_ctrl.borrow_mut().remove_listener("foo");
    /// ```
    pub fn remove_listener(&mut self, id: &str) {
        let _ = self.callbacks.remove(id);
    }

    /// Waits for events and notifies all listeners.
    ///
    /// # Errors
    ///
    /// Returns an error if the channel has hang up.
    ///
    /// # Example
    /// ```ignore
    /// use crossterm_events::EventController;
    ///
    /// struct AppState;
    /// struct AppEvent;
    ///
    /// let mut state = AppState;
    /// let mut event_ctrl = EventController::<AppState, AppEvent>::new();
    ///
    /// event_ctrl
    ///     .borrow_mut()
    ///     .handle_events(&mut state)
    ///     .expect("failed to handle events");
    /// ```
    pub fn handle_events(&self, state: &mut S) -> Result<()> {
        let event = self.receiver.recv()?;
        self.notify_listener(state, &event);

        Ok(())
    }

    /// Notify all listener.
    fn notify_listener(&self, state: &mut S, event: &E) {
        for callback in self.callbacks.values() {
            (callback)(state, event);
        }
    }
}

/// A trait for widgets that can handle events.
pub trait EventListener<S, E> {
    /// Returns a unique key for identifying the widget in the event controller.
    fn key() -> String;

    /// Handles incoming events for the widget.
    ///
    /// # Arguments
    /// - `state`: A mutable reference to the state associated with the widget.
    /// - `event`: The event to be processed.
    /// - `area`: The area of the widget from the last render.
    fn handle_event(state: &mut S, event: &E, area: Option<Rect>);
}

/// A wrapper for a widget that integrates with an event controller.
///
/// This automatically registers a listener on the `EventController`
/// when constructed with `EventfulWidget::new` and removes the listener
/// when it is dropped.
pub struct EventfulWidget<S, E, W>
where
    W: EventListener<S, E>,
{
    widget: W,
    ctrl: Rc<RefCell<EventController<S, E>>>,
    area: Rc<RefCell<Option<Rect>>>,
}

impl<S, E, W> EventfulWidget<S, E, W>
where
    S: 'static,
    E: 'static,
    W: EventListener<S, E>,
    for<'a> &'a W: Widget,
{
    /// Creates a new instance of `EventfulWidget` and registers a
    /// callback on the the event controller.
    #[must_use]
    pub fn new(widget: W, controller: &Rc<RefCell<EventController<S, E>>>) -> Self {
        let area = Rc::new(RefCell::new(None));
        let area_clone = Rc::clone(&area);

        let ctrl = Rc::clone(controller);

        let key = &W::key();
        ctrl.borrow_mut().add_listener(key, move |state, event| {
            let area = area_clone.borrow();
            W::handle_event(state, event, *area);
        });

        Self { widget, ctrl, area }
    }
}

impl<S, E, W> Drop for EventfulWidget<S, E, W>
where
    W: EventListener<S, E>,
{
    /// Removes the listener from the event controller when it is dropped.
    fn drop(&mut self) {
        let key = &W::key();
        self.ctrl.borrow_mut().remove_listener(key);
    }
}

impl<S, E, W> Widget for &mut EventfulWidget<S, E, W>
where
    W: EventListener<S, E>,
    for<'a> &'a W: Widget,
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        *self.area.borrow_mut() = Some(area);
        self.widget.render(area, buf);
    }
}
