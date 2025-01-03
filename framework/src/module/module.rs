use std::collections::{HashMap, HashSet};

use async_cron_scheduler::cron::Schedule;
use twilight_gateway::EventType;
use twilight_model::application::command::Command;

use crate::handler::{
    command_handler::{CommandFunc, CommandHandler},
    component_interaction_handler::{ComponentInteractionFunc, ComponentInteractionHandler},
    event_handler::{EventFunc, EventHandler},
    task_handler::{TaskFunc, TaskHandler},
};

pub struct ModuleBuilder<T: Clone> {
    name: String,
    guild_scoped: bool,

    commands: HashMap<String, CommandHandler<T>>,
    components: HashMap<String, ComponentInteractionHandler<T>>,
    events: HashMap<EventType, HashSet<EventHandler<T>>>,
    tasks: HashMap<String, TaskHandler<T>>,
}

impl<T: Clone> ModuleBuilder<T> {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            guild_scoped: false,

            commands: HashMap::new(),
            components: HashMap::new(),
            events: HashMap::new(),
            tasks: HashMap::new(),
        }
    }

    pub fn build(self) -> Module<T> {
        Module {
            name: self.name,
            guild_scoped: self.guild_scoped,

            commands: self.commands,
            components: self.components,
            events: self.events,
            tasks: self.tasks,
        }
    }

    pub fn guild(mut self) -> Self {
        self.guild_scoped = true;
        self
    }

    pub fn command(mut self, definition: Command, func: CommandFunc<T>) -> Self {
        self.commands.insert(
            definition.name.clone(),
            CommandHandler {
                module: self.name.clone(),
                definition,
                func,
            },
        );
        self
    }

    pub fn component(mut self, custom_id: &str, func: ComponentInteractionFunc<T>) -> Self {
        self.components.insert(
            custom_id.to_string(),
            ComponentInteractionHandler {
                module: self.name.clone(),
                custom_id: custom_id.to_string(),
                func,
            },
        );
        self
    }

    pub fn event(mut self, event: EventType, func: EventFunc<T>) -> Self {
        self.events
            .entry(event)
            .or_default()
            .insert(EventHandler {
                module: self.name.clone(),
                uuid: uuid::Uuid::now_v7().to_string(),
                event,
                func,
            });
        self
    }

    pub fn task(mut self, name: &str, schedule: &str, func: TaskFunc<T>) -> Self {
        self.tasks.insert(
            name.to_string(),
            TaskHandler {
                module: self.name.clone(),
                name: name.to_string(),
                cron: Schedule::try_from(schedule).expect("failed to parse cron expression"),
                func,
            },
        );
        self
    }
}
