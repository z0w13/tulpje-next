use std::sync::Arc;

use tulpje_shared::DiscordEventMeta;
use twilight_http::{client::InteractionClient, response::marker::EmptyBody, Client};
use twilight_model::{
    application::interaction::message_component::MessageComponentInteractionData,
    gateway::payload::incoming::InteractionCreate,
    guild::Guild,
    http::interaction::InteractionResponse,
    id::{marker::ApplicationMarker, Id},
};

use crate::Error;

#[derive(Clone, Debug)]
pub struct ComponentInteractionContext<T: Clone + Send + Sync> {
    pub meta: DiscordEventMeta,
    pub application_id: Id<ApplicationMarker>,
    pub services: T,
    pub client: Arc<Client>,

    pub event: InteractionCreate,
    pub interaction: MessageComponentInteractionData,
}

impl<T: Clone + Send + Sync> ComponentInteractionContext<T> {
    pub fn interaction(&self) -> InteractionClient<'_> {
        self.client.interaction(self.application_id)
    }

    pub async fn guild(&self) -> Result<Option<Guild>, Error> {
        let Some(guild_id) = self.event.guild_id else {
            return Ok(None);
        };

        Ok(Some(self.client.guild(guild_id).await?.model().await?))
    }

    pub async fn response(
        &self,
        response: InteractionResponse,
    ) -> Result<twilight_http::Response<EmptyBody>, twilight_http::Error> {
        self.interaction()
            .create_response(self.event.id, &self.event.token, &response)
            .await
    }
}
