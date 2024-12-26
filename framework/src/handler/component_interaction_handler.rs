use std::{error::Error, future::Future, pin::Pin};

use super::super::context::ComponentInteractionContext;

use super::InteractionHandler;

type ComponentInteractionFunc<T> =
    fn(
        ComponentInteractionContext<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>>>>;

pub struct ComponentInteractionHandler<T: Clone> {
    pub custom_id: String,
    pub func: ComponentInteractionFunc<T>,
}

impl<T: Clone> InteractionHandler<String> for ComponentInteractionHandler<T> {
    fn key(&self) -> String {
        self.custom_id.clone()
    }
}

impl<T: Clone> ComponentInteractionHandler<T> {
    pub async fn run(&self, ctx: ComponentInteractionContext<T>) -> Result<(), Box<dyn Error>> {
        // can add more handling/parsing/etc here in the future
        (self.func)(ctx).await
    }
}
