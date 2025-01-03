use std::sync::Arc;

use tulpje_shared::DiscordEventMeta;
use twilight_gateway::Event;
use twilight_http::Client;
use twilight_model::id::{marker::ApplicationMarker, Id};

#[derive(Clone, Debug)]
pub struct EventContext<T: Clone + Send + Sync> {
    pub meta: DiscordEventMeta,
    pub application_id: Id<ApplicationMarker>,
    pub services: T,
    pub client: Arc<Client>,

    pub event: Event,
}
