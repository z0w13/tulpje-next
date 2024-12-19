use std::sync::Arc;

use bb8_redis::RedisConnectionManager;
use tulpje_shared::DiscordEventMeta;
use twilight_http::{client::InteractionClient, response::marker::EmptyBody, Client};
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate,
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::{marker::ApplicationMarker, Id},
};
use twilight_util::builder::InteractionResponseDataBuilder;

#[derive(Clone, Debug)]
pub struct Services {
    pub redis: bb8::Pool<RedisConnectionManager>,
}

#[derive(Debug)]
pub struct Context {
    pub application_id: Id<ApplicationMarker>,
    pub services: Services,
    pub client: Client,
}

impl Context {
    pub fn interaction(&self) -> InteractionClient<'_> {
        self.client.interaction(self.application_id)
    }
}

pub struct CommandContext {
    pub meta: DiscordEventMeta,
    pub context: Arc<Context>,
    pub command: CommandData,
    pub event: InteractionCreate,
}

impl CommandContext {
    pub fn interaction(&self) -> InteractionClient<'_> {
        self.context.client.interaction(self.context.application_id)
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
