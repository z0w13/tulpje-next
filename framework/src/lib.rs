use std::sync::Arc;

use tulpje_shared::DiscordEventMeta;
use twilight_gateway::Event;
use twilight_model::gateway::payload::incoming::InteractionCreate;

pub use context::{Context, EventContext, InteractionContext};
pub use module::{builder::ModuleBuilder, registry::Registry, Module};
pub use scheduler::Scheduler;

pub mod context;
pub mod handler;
pub mod interaction;
pub mod macros;
pub mod module;
pub mod scheduler;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn handle_interaction<T: Clone + Send + Sync + 'static>(
    event: InteractionCreate,
    context: Context<T>,
    meta: &DiscordEventMeta,
    registry: &Registry<T>,
) -> Result<(), Error> {
    tracing::info!("interaction");

    match interaction::parse(&event, meta.clone(), context) {
        Ok(InteractionContext::Command(ctx)) => {
            let Some(command) = registry.find_command(&ctx.command.name) else {
                return Err(format!("unknown command /{}", ctx.command.name).into());
            };

            if let Err(err) = command.run(ctx.clone()).await {
                return Err(format!("error running command /{}: {}", ctx.command.name, err).into());
            }
        }
        Ok(InteractionContext::ComponentInteraction(ctx)) => {
            let Some(component_interaction) = registry.components.get(&ctx.interaction.custom_id)
            else {
                return Err(format!(
                    "no handler for component interaction {}",
                    ctx.interaction.custom_id
                )
                .into());
            };

            if let Err(err) = component_interaction.run(ctx.clone()).await {
                return Err(format!(
                    "error handling component interaction {}: {}",
                    ctx.interaction.custom_id, err
                )
                .into());
            }
        }
        Ok(InteractionContext::Modal(_modal_context)) => {
            todo!()
        }
        Err(err) => return Err(format!("error handling interaction: {}", err).into()),
    };

    Ok(())
}

pub async fn handle<T: Clone + Send + Sync + 'static>(
    meta: DiscordEventMeta,
    ctx: Context<T>,
    registry: &Registry<T>,
    event: Event,
) {
    match event.clone() {
        twilight_gateway::Event::InteractionCreate(event) => {
            if let Err(err) = handle_interaction(*event, ctx.clone(), &meta, registry).await {
                tracing::warn!(err);
            }
        }
        // TODO: Better tracking of this, if there's no module or interaction handlers?
        e => tracing::warn!(event = ?e.kind(), "unhandled event"),
    }

    if let Some(handlers) = registry.events.get(&event.kind()) {
        tracing::info!("running event handlers for {:?}", event.kind());

        for handler in handlers {
            let event_ctx = EventContext {
                meta: meta.clone(),
                application_id: ctx.application_id,
                client: Arc::clone(&ctx.client),
                services: ctx.services.clone(),

                event: event.clone(),
            };

            if let Err(err) = handler.run(event_ctx).await {
                tracing::warn!("error running event handler {}: {}", handler.uuid, err);
            }
        }
    }
}
