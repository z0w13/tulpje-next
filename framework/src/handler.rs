pub mod autocomplete_handler;
pub mod command_handler;
pub mod component_interaction_handler;
pub mod event_handler;
pub mod modal_handler;
pub mod task_handler;

pub trait InteractionHandler<T> {
    fn key(&self) -> T;
}
