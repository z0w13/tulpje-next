use std::sync::Arc;

use tulpje_shared::DiscordEventMeta;
use twilight_http::{client::InteractionClient, response::marker::EmptyBody, Client};
use twilight_model::{
    application::interaction::application_command::{CommandData, CommandOptionValue},
    channel::{message::MessageFlags, Message},
    gateway::payload::incoming::InteractionCreate,
    guild::Guild,
    http::interaction::{InteractionResponse, InteractionResponseType},
    id::{marker::ApplicationMarker, Id},
};
use twilight_util::builder::InteractionResponseDataBuilder;

use super::Context;
use crate::Error;

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
    pub fn from_context(
        meta: DiscordEventMeta,
        ctx: Context<T>,
        event: InteractionCreate,
        command: CommandData,
    ) -> Self {
        Self {
            meta,
            application_id: ctx.application_id,
            client: ctx.client,
            services: ctx.services,

            command,
            event,
        }
    }

    pub fn interaction(&self) -> InteractionClient<'_> {
        self.client.interaction(self.application_id)
    }

    pub fn client(&self) -> Arc<Client> {
        self.client.clone()
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

    pub async fn update(
        &self,
        message: impl Into<String>,
    ) -> Result<twilight_http::Response<Message>, twilight_http::Error> {
        self.interaction()
            .update_response(&self.event.token)
            .content(Some(&message.into()))
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

    pub async fn defer(&self) -> Result<twilight_http::Response<EmptyBody>, twilight_http::Error> {
        self.response(InteractionResponse {
            kind: InteractionResponseType::DeferredChannelMessageWithSource,
            data: None,
        })
        .await
    }
    pub async fn defer_ephemeral(
        &self,
    ) -> Result<twilight_http::Response<EmptyBody>, twilight_http::Error> {
        self.response(InteractionResponse {
            kind: InteractionResponseType::DeferredChannelMessageWithSource,
            data: Some(
                InteractionResponseDataBuilder::new()
                    .flags(MessageFlags::EPHEMERAL)
                    .build(),
            ),
        })
        .await
    }

    pub fn get_arg_string_optional(&self, name: &str) -> Result<Option<String>, Error> {
        let Some(opt) = self.command.options.iter().find(|opt| opt.name == name) else {
            return Ok(None);
        };

        let CommandOptionValue::String(value) = &opt.value else {
            return Err(format!("option '{}' not a string option", name).into());
        };

        Ok(Some(value.clone()))
    }

    pub fn get_arg_string(&self, name: &str) -> Result<String, Error> {
        if let Some(value) = self.get_arg_string_optional(name)? {
            Ok(value)
        } else {
            Err(format!("couldn't find command argument {}", name).into())
        }
    }
}
