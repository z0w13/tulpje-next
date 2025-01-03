use std::{future::Future, pin::Pin};

use twilight_model::application::command::Command;

use super::super::context::CommandContext;

use super::InteractionHandler;
use crate::Error;

pub(crate) type CommandFunc<T> =
    fn(CommandContext<T>) -> Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>;

#[derive(Clone)]
pub struct CommandHandler<T: Clone> {
    pub module: String,
    pub definition: Command,
    pub func: CommandFunc<T>,
}

impl<T: Clone> InteractionHandler<String> for CommandHandler<T> {
    fn key(&self) -> String {
        self.definition.name.clone()
    }
}

impl<T: Clone> CommandHandler<T> {
    pub async fn run(&self, ctx: CommandContext<T>) -> Result<(), Error> {
        // can add more handling/parsing/etc here in the future
        (self.func)(ctx).await
    }
}
