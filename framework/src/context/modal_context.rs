use std::sync::Arc;

use twilight_http::Client;
use twilight_model::{
    application::interaction::modal::ModalInteractionData,
    gateway::payload::incoming::InteractionCreate,
    id::{marker::ApplicationMarker, Id},
};

use tulpje_shared::DiscordEventMeta;

#[derive(Clone, Debug)]
pub struct ModalContext<T: Clone + Send + Sync> {
    pub meta: DiscordEventMeta,
    pub application_id: Id<ApplicationMarker>,
    pub services: T,
    pub client: Arc<Client>,

    pub event: InteractionCreate,
    pub data: ModalInteractionData,
}
