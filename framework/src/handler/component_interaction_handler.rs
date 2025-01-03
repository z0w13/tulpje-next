use std::{future::Future, pin::Pin};

use super::super::context::ComponentInteractionContext;
use super::InteractionHandler;
use crate::Error;

pub(crate) type ComponentInteractionFunc<T> =
    fn(ComponentInteractionContext<T>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>;

#[derive(Clone)]
pub struct ComponentInteractionHandler<T: Clone> {
    pub module: String,
    pub custom_id: String,
    pub func: ComponentInteractionFunc<T>,
}

impl<T: Clone> InteractionHandler<String> for ComponentInteractionHandler<T> {
    fn key(&self) -> String {
        self.custom_id.clone()
    }
}

impl<T: Clone> ComponentInteractionHandler<T> {
    pub async fn run(&self, ctx: ComponentInteractionContext<T>) -> Result<(), Error> {
        // can add more handling/parsing/etc here in the future
        (self.func)(ctx).await
    }
}
