use std::sync::Arc;

use twilight_http::{client::InteractionClient, Client};
use twilight_model::id::{marker::ApplicationMarker, Id};

pub mod autocomplete_context;
pub mod command_context;
pub mod component_interaction_context;
pub mod event_context;
pub mod modal_context;
pub mod task_context;

pub use command_context::CommandContext;
pub use component_interaction_context::ComponentInteractionContext;
pub use event_context::EventContext;
pub use modal_context::ModalContext;
pub use task_context::TaskContext;

#[derive(Debug)]
pub struct Context<T: Clone + Send + Sync> {
    pub application_id: Id<ApplicationMarker>,
    pub services: T,
    pub client: Arc<Client>,
}

impl<T: Clone + Send + Sync> Context<T> {
    pub fn interaction(&self) -> InteractionClient<'_> {
        self.client.interaction(self.application_id)
    }
}

impl<T: Clone + Send + Sync> Clone for Context<T> {
    fn clone(&self) -> Self {
        Self {
            application_id: self.application_id,
            services: self.services.clone(),
            client: Arc::clone(&self.client),
        }
    }
}

pub enum InteractionContext<T: Clone + Send + Sync> {
    Command(CommandContext<T>),
    ComponentInteraction(ComponentInteractionContext<T>),
    Modal(ModalContext<T>),
}
