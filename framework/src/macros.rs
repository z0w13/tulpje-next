#[macro_export]
macro_rules! handler_func {
    ($func:expr $(,)?) => {
        |ctx| Box::pin($func(ctx))
    };
}
