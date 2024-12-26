use std::{
    collections::{hash_map::Values, HashMap, HashSet},
    hash::Hash,
};

use twilight_gateway::EventType;
use twilight_model::application::command::Command;

use super::handler::{
    command_handler::CommandHandler, component_interaction_handler::ComponentInteractionHandler,
    event_handler::EventHandler, InteractionHandler,
};

pub struct InteractionRegistry<K: Eq + Hash, T: InteractionHandler<K>> {
    interactions: HashMap<K, T>,
}

impl<K: Eq + Hash, T: InteractionHandler<K>> InteractionRegistry<K, T> {
    pub fn new() -> Self {
        Self {
            interactions: HashMap::new(),
        }
    }

    pub fn values(&self) -> Values<'_, K, T> {
        self.interactions.values()
    }

    pub fn insert(&mut self, val: T) -> Option<T> {
        self.interactions.insert(val.key(), val)
    }

    pub fn remove(&mut self, val: T) -> Option<T> {
        self.interactions.remove(&val.key())
    }

    pub fn get(&mut self, key: &K) -> Option<&T> {
        self.interactions.get(key)
    }
}

pub struct EventRegistry<T: Clone> {
    handlers: HashMap<EventType, HashSet<EventHandler<T>>>,
}

impl<T: Clone> EventRegistry<T> {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn insert(&mut self, val: EventHandler<T>) {
        if let Some(handlers) = self.handlers.get_mut(&val.event) {
            handlers.insert(val);
        } else {
            self.handlers.insert(val.event, [val].into());
        }
    }

    pub fn remove(&mut self, val: EventHandler<T>) -> bool {
        if let Some(handlers) = self.handlers.get_mut(&val.event) {
            handlers.remove(&val)
        } else {
            false
        }
    }

    pub fn get_all(&mut self, key: EventType) -> Option<Vec<&EventHandler<T>>> {
        self.handlers.get(&key).map(|set| set.iter().collect())
    }
}

pub struct Registry<T: Clone> {
    pub command: InteractionRegistry<String, CommandHandler<T>>,
    pub component_interaction: InteractionRegistry<String, ComponentInteractionHandler<T>>,
    pub event: EventRegistry<T>,
    // pub autocomplete: InteractionRegistry<AutocompleteHandler<T>>,
    // pub modal: InteractionRegistry<ModalHandler<T>>,
}

impl<T: Clone> Registry<T> {
    pub fn new() -> Self {
        Self {
            command: InteractionRegistry::<String, CommandHandler<T>>::new(),
            component_interaction:
                InteractionRegistry::<String, ComponentInteractionHandler<T>>::new(),
            event: EventRegistry::<T>::new(),
        }
    }

    pub fn get_global_commands(&self) -> Vec<Command> {
        self.command
            .values()
            .map(|cmd| cmd.definition.clone())
            .collect()
    }
}
