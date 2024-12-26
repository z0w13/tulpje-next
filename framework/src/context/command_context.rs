use std::{error::Error, sync::Arc};

use tulpje_shared::DiscordEventMeta;
use twilight_http::{client::InteractionClient, response::marker::EmptyBody, Client};
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate,
    guild::Guild,
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::{marker::ApplicationMarker, Id},
};
use twilight_util::builder::InteractionResponseDataBuilder;

#[derive(Clone, Debug)]
pub struct CommandContext<T: Clone> {
    pub meta: DiscordEventMeta,
    pub application_id: Id<ApplicationMarker>,
    pub services: T,
    pub client: Arc<Client>,

    pub event: InteractionCreate,
    pub command: CommandData,
}

impl<T: Clone> CommandContext<T> {
    pub fn interaction(&self) -> InteractionClient<'_> {
        self.client.interaction(self.application_id)
    }

    pub fn client(&self) -> Arc<Client> {
        self.client.clone()
    }

    pub async fn guild(&self) -> Result<Option<Guild>, Box<dyn Error>> {
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

    pub async fn reply(
        &self,
        message: impl Into<String>,
    ) -> Result<twilight_http::Response<EmptyBody>, twilight_http::Error> {
        let response = InteractionResponseDataBuilder::new()
            .content(message)
            .build();

        self.response(InteractionResponse {
            kind: InteractionResponseType::ChannelMessageWithSource,
            data: Some(response),
        })
        .await
    }
}
