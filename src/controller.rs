use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::mpsc;

/// Manages events and their associated callbacks in a Ratatui application.
///
/// # Type Parameters
/// - `S`: The type representing the application state.
/// - `E`: The type representing the events.
#[derive(Clone)]
pub struct EventController<S, E> {
    rc: Rc<RefCell<InternalEventController<S, E>>>,
}

impl<S, E> EventController<S, E> {
    /// Creates a new [`EventController`].
    #[must_use]
    pub fn new() -> Self {
        let controller = InternalEventController::new();

        Self {
            rc: Rc::new(RefCell::new(controller)),
        }
    }

    /// Registers an event listener under the specified `id`.
    ///
    /// The `id` must be unique, as there can only be one callback associated
    /// with a given `id`. If a callback with the same `id` already exists,
    /// it will be replaced.
    ///
    /// # Note
    /// Ensure to remove the callback when it is no longer needed using
    /// [`Self::remove_listener`].
    ///
    /// # Example
    /// ```
    /// type EventController = tui_event_controller::EventController<AppState, AppEvent>;
    ///
    /// #[derive(Debug)]
    /// struct AppState;
    ///
    /// #[derive(Debug)]
    /// struct AppEvent;
    ///
    /// let controller = EventController::new();
    /// controller.add_listener("foo", move |ctx, _state| {
    ///     println!("received: {:?}", ctx.event);
    /// });
    /// ```
    pub fn add_listener<F>(&self, id: &str, callback: F)
    where
        F: Fn(EventContext<S, E>, &mut S) + 'static,
    {
        self.rc.borrow_mut().add_listener(id, callback);
    }

    /// Removes a listener with a given `id`.
    ///
    /// It is safe to call this method even if there is no listener
    /// registered with a given `id`.
    ///
    /// # Example
    /// ```
    /// type EventController = tui_event_controller::EventController<AppState, AppEvent>;
    ///
    /// struct AppState;
    /// struct AppEvent;
    ///
    /// let controller = EventController::new();
    /// controller.remove_listener("foo");
    /// ```
    pub fn remove_listener(&self, id: &str) {
        self.rc.borrow_mut().remove_listener(id);
    }

    /// Returns a clone of the event sender.
    ///
    /// This allows to send events to the controller.
    pub fn get_sender(&self) -> std::sync::mpsc::Sender<E> {
        self.rc.borrow().sender.clone()
    }

    /// Waits for an events and sends the event to all listeners.
    ///
    /// This function will block the current thread until an event
    /// is received. Once the message is receveived, all listeners
    /// that registered with [`Self::add_listener`] are notified.
    ///
    /// # Errors
    ///
    /// Returns an error if the channel has hang up.
    ///
    /// # Example
    /// ```ignore
    /// use tui_event_controller::{EventController, handle_events};
    ///
    /// struct AppState;
    /// struct AppEvent;
    ///
    /// let mut state = AppState;
    /// let mut event_ctrl = EventController::<AppState, AppEvent>::new();
    ///
    ///
    /// recv_and_notify(&event_ctrl, &mut state).unwrap();
    /// ```
    pub fn recv_and_notify(&self, state: &mut S) -> Result<(), mpsc::RecvError> {
        let event = self.rc.borrow().receiver.recv()?;

        let callbacks = self.rc.borrow().callbacks.clone();
        for callback in callbacks.values() {
            let ctx = EventContext {
                controller: self,
                event: &event,
            };
            (callback)(ctx, state);
        }

        Ok(())
    }

    /// Returns an [`Rc::clone`]d instance of the `EventController`.
    pub(super) fn rc_clone(&self) -> Self {
        Self {
            rc: Rc::clone(&self.rc),
        }
    }
}

/// Provides event-related data to callbacks.
///
/// - `controller`: A reference to the [`EventController`].
/// - `event`: The event being processed.
pub struct EventContext<'a, S, E> {
    pub controller: &'a EventController<S, E>,
    pub event: &'a E,
}

type EventCallback<S, E> = Rc<dyn Fn(EventContext<S, E>, &mut S) + 'static>;

/// The internal implementation of `EventController`.
struct InternalEventController<S, E> {
    /// Event sender channel.
    sender: mpsc::Sender<E>,

    /// Event receiver channel.
    receiver: mpsc::Receiver<E>,

    /// Registered callbacks.
    callbacks: HashMap<String, EventCallback<S, E>>,
}

impl<S, E> InternalEventController<S, E> {
    /// Creates a new instance of [`EventHandler`].
    #[must_use]
    fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let callbacks = HashMap::default();

        Self {
            sender,
            receiver,
            callbacks,
        }
    }

    /// Adds a new listener.
    fn add_listener<F>(&mut self, id: &str, callback: F)
    where
        F: Fn(EventContext<S, E>, &mut S) + 'static,
    {
        self.callbacks.insert(id.to_string(), Rc::new(callback));
    }

    /// Removes a listener.
    fn remove_listener(&mut self, id: &str) {
        let _ = self.callbacks.remove(id);
    }
}
