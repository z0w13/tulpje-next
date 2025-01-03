use std::hash::{Hash, Hasher};
use std::{future::Future, pin::Pin};

use twilight_gateway::EventType;

use super::super::context::EventContext;
use crate::Error;

pub(crate) type EventFunc<T> =
    fn(EventContext<T>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>;

#[derive(Clone)]
pub struct EventHandler<T: Clone> {
    pub module: String,
    pub uuid: String,
    pub event: EventType,
    pub func: EventFunc<T>,
}

impl<T: Clone> EventHandler<T> {
    pub async fn run(&self, ctx: EventContext<T>) -> Result<(), Error> {
        // can add more handling/parsing/etc here in the future
        (self.func)(ctx).await
    }
}

impl<T: Clone> Hash for EventHandler<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.uuid.hash(state);
        self.event.hash(state);
    }
}

impl<T: Clone> PartialEq for EventHandler<T> {
    fn eq(&self, other: &Self) -> bool {
        self.event == other.event && self.uuid == other.uuid
    }
}

impl<T: Clone> Eq for EventHandler<T> {}
