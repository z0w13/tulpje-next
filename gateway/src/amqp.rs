#[cfg(all(feature = "amqp-amqprs", feature = "amqp-lapin"))]
compile_error!(
    "can only pick one amqp implementation, `amqp-amqprs` and `amqp-lapin` are mutually exclusive"
);

#[cfg(feature = "amqp-amqprs")]
mod amqprs;
#[cfg(feature = "amqp-lapin")]
mod lapin;

#[cfg(feature = "amqp-amqprs")]
pub(crate) use amqprs::create;

#[cfg(feature = "amqp-lapin")]
pub(crate) use lapin::create;
