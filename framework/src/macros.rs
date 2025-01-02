#[macro_export]
macro_rules! handler_func {
    ($func:expr $(,)?) => {
        |ctx| Box::pin($func(ctx))
    };
}

#[macro_export]
macro_rules! command {
    ($reg:expr, $def:expr, $func:expr $(,)?) => {
        $reg.command.insert(CommandHandler {
            definition: $def,
            func: |ctx| Box::pin($func(ctx)),
        });
    };
}

#[macro_export]
macro_rules! guild_command {
    ($reg:expr, $group:expr, $def:expr, $func:expr $(,)?) => {
        $reg.guild_command.insert($group.into(), CommandHandler {
            definition: $def,
            func: |ctx| Box::pin($func(ctx)),
        });
    };
}

#[macro_export]
macro_rules! event_handler {
    ($reg:expr, $event:expr, $func:expr $(,)?) => {
        $reg.event.insert(EventHandler {
            uuid: uuid::Uuid::now_v7().to_string(),
            event: $event,
            func: |ctx| Box::pin($func(ctx)),
        });
    };
}

#[macro_export]
macro_rules! component_interaction {
    ($reg:expr, $id:expr, $func:expr $(,)?) => {
        $reg.component_interaction
            .insert(ComponentInteractionHandler {
                custom_id: $id.into(),
                func: |ctx| Box::pin($func(ctx)),
            })
    };
}

#[macro_export]
macro_rules! task {
    ($reg:expr, $name:expr, $schedule:expr, $func:expr $(,)?) => {
        $reg.task
            .insert(TaskHandler {
                name: $name.into(),
                cron: async_cron_scheduler::cron::Schedule::try_from($schedule)
                    .expect("failed to parse cron expression"),
                func: |ctx| Box::pin($func(ctx)),
            })
            .await
    };
}
