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
            func: handler_func!($func),
        });
    };
}

#[macro_export]
macro_rules! event_handler {
    ($reg:expr, $event:expr, $func:expr $(,)?) => {
        $reg.event.insert(EventHandler {
            uuid: uuid::Uuid::now_v7().to_string(),
            event: $event,
            func: handler_func!($func),
        });
    };
}

#[macro_export]
macro_rules! component_interaction {
    ($reg:expr, $id:expr, $func:expr $(,)?) => {
        $reg.component_interaction
            .insert(ComponentInteractionHandler {
                custom_id: $id.into(),
                func: handler_func!($func),
            })
    };
}
