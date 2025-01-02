use std::{
    collections::{hash_map::Values, HashMap, HashSet},
    hash::Hash,
};

use async_cron_scheduler::{Job, JobId, Scheduler};
use chrono::Utc;
use twilight_gateway::EventType;
use twilight_model::application::command::Command;

use crate::context::{Context, TaskContext};

use super::handler::{
    command_handler::CommandHandler, component_interaction_handler::ComponentInteractionHandler,
    event_handler::EventHandler, task_handler::TaskHandler, InteractionHandler,
};

pub struct InteractionRegistry<K: Eq + Hash, T: InteractionHandler<K>> {
    interactions: HashMap<K, T>,
}

impl<K: Eq + Hash, T: InteractionHandler<K>> InteractionRegistry<K, T> {
    #[expect(
        clippy::new_without_default,
        reason = "we might have constructor arguments in the future, having a Default implementation feels incorrect"
    )]
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

    pub fn get(&self, key: &K) -> Option<&T> {
        self.interactions.get(key)
    }
}

pub struct EventRegistry<T: Clone> {
    handlers: HashMap<EventType, HashSet<EventHandler<T>>>,
}

impl<T: Clone> EventRegistry<T> {
    #[expect(
        clippy::new_without_default,
        reason = "we might have constructor arguments in the future, having a Default implementation feels incorrect"
    )]
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

pub struct TaskService<T: Clone> {
    handlers: HashMap<String, TaskHandler<T>>,
    job_map: HashMap<String, JobId>,
    scheduler: Option<Scheduler<Utc>>,
}

impl<T: Clone + Send + Sync + 'static> TaskService<T> {
    #[expect(
        clippy::new_without_default,
        reason = "we might have constructor arguments in the future, having a Default implementation feels incorrect"
    )]
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            job_map: HashMap::new(),
            scheduler: None,
        }
    }

    pub async fn insert(&mut self, handler: TaskHandler<T>) {
        if self.scheduler.is_some() {
            panic!("trying to insert a task while scheduler is already running");
        }

        self.handlers.insert(handler.name.clone(), handler.clone());
    }

    pub async fn remove(&mut self, name: &str) -> bool {
        let existed = self.handlers.remove(name).is_some();
        let job_id = self.job_map.remove(name);

        // immediately return if the scheduler wasn't started yet
        let Some(ref mut scheduler) = self.scheduler else {
            return existed;
        };

        if let Some(job_id) = job_id {
            scheduler.remove(job_id).await
        }

        existed
    }

    pub async fn run(&mut self, ctx: Context<T>) -> tokio::task::JoinHandle<()> {
        let (mut scheduler, sched_service) = Scheduler::<Utc>::launch(tokio::time::sleep);

        for handler in self.handlers.values() {
            let job_id = insert_job(&mut scheduler, handler.clone(), ctx.clone()).await;
            self.job_map.insert(handler.name.clone(), job_id);
        }

        self.scheduler = Some(scheduler);
        tokio::spawn(sched_service)
    }
}

async fn insert_job<T: Clone + Send + Sync + 'static>(
    sched: &mut Scheduler<Utc>,
    handler: TaskHandler<T>,
    ctx: Context<T>,
) -> JobId {
    let job = Job::cron_schedule(handler.cron.clone());
    sched
        .insert(job, move |_id| {
            let job_ctx = ctx.clone();
            let job_handler = handler.clone();

            tokio::spawn(async move {
                if let Err(err) = job_handler.run(TaskContext::from_context(job_ctx)).await {
                    tracing::error!("error running task {}: {}", job_handler.name, err);
                };
            });
        })
        .await
}

pub struct GuildCommandRegistry<T: Clone + Send + Sync> {
    pub groups: HashMap<String, InteractionRegistry<String, CommandHandler<T>>>,
}

impl<T: Clone + Send + Sync + 'static> GuildCommandRegistry<T> {
    #[expect(
        clippy::new_without_default,
        reason = "we might have constructor arguments in the future, having a Default implementation feels incorrect"
    )]
    pub fn new() -> Self {
        Self {
            groups: HashMap::new(),
        }
    }

    pub fn insert(&mut self, group: String, handler: CommandHandler<T>) {
        if !self.groups.contains_key(&group) {
            self.groups
                .insert(group.clone(), InteractionRegistry::new());
        }

        self.groups.get_mut(&group).unwrap().insert(handler);
    }

    pub fn get_group(&self, key: &String) -> Option<Vec<&CommandHandler<T>>> {
        self.groups.get(key).map(|v| v.values().collect())
    }

    pub fn find_command(&self, name: &String) -> Option<&CommandHandler<T>> {
        self.groups.values().find_map(|r| r.get(name))
    }

    pub fn get_definitions(&self) -> HashMap<String, Vec<Command>> {
        self.groups
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.values().map(|v| v.definition.clone()).collect(),
                )
            })
            .collect()
    }
}

pub struct Registry<T: Clone + Send + Sync> {
    pub command: InteractionRegistry<String, CommandHandler<T>>,
    pub guild_command: GuildCommandRegistry<T>,
    pub component_interaction: InteractionRegistry<String, ComponentInteractionHandler<T>>,
    pub event: EventRegistry<T>,
    pub task: TaskService<T>,
    // pub autocomplete: InteractionRegistry<AutocompleteHandler<T>>,
    // pub modal: InteractionRegistry<ModalHandler<T>>,
}

impl<T: Clone + Send + Sync + 'static> Registry<T> {
    #[expect(
        clippy::new_without_default,
        reason = "we might have constructor arguments in the future, having a Default implementation feels incorrect"
    )]
    pub fn new() -> Self {
        Self {
            command: InteractionRegistry::<String, CommandHandler<T>>::new(),
            guild_command: GuildCommandRegistry::new(),
            component_interaction:
                InteractionRegistry::<String, ComponentInteractionHandler<T>>::new(),
            event: EventRegistry::<T>::new(),
            task: TaskService::<T>::new(),
        }
    }

    pub fn find_command(&self, name: &String) -> Option<&CommandHandler<T>> {
        if let Some(command) = self.command.get(name) {
            return Some(command);
        }

        self.guild_command.find_command(name)
    }

    pub fn get_global_commands(&self) -> Vec<Command> {
        self.command
            .values()
            .map(|cmd| cmd.definition.clone())
            .collect()
    }
}
