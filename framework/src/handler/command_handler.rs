use std::{error::Error, future::Future, pin::Pin};

use twilight_model::application::command::Command;

use super::super::context::CommandContext;

use super::InteractionHandler;

type CommandFunc<T> =
    fn(CommandContext<T>) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>>>>;

pub struct CommandHandler<T: Clone> {
    pub definition: Command,
    pub func: CommandFunc<T>,
}

impl<T: Clone> InteractionHandler<String> for CommandHandler<T> {
    fn key(&self) -> String {
        self.definition.name.clone()
    }
}

impl<T: Clone> CommandHandler<T> {
    pub async fn run(&self, ctx: CommandContext<T>) -> Result<(), Box<dyn Error>> {
        // can add more handling/parsing/etc here in the future
        (self.func)(ctx).await
    }
}
