use std::sync::mpsc::{Sender, Receiver, channel};

pub trait Event<C> {
    fn handle(&self, context: &mut C);
}

pub struct EventHandler<E: Event, C> {
    event_sender: Sender<E>,
    event_receiver: Receiver<E>,
    context: C
}

impl EventHandler<E, C> {
    pub fn new(context: C) -> Self {
        let (tx, rx) = channel();

        EventHandler {
            event_sender: tx,
            event_receiver: rx,
            context
        }
    }

    pub fn run_event_loop(&self) {
        for event in self.event_receiver {
            event.handle(&self);
        }
    }

    pub fn event_sender(&self) -> Sender<E> {
        self.event_sender.clone()
    }
}