use std::error::Error;

use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{BasicPublishArguments, Channel, QueueDeclareArguments},
    connection::{Connection, OpenConnectionArguments},
    BasicProperties,
};

pub(crate) struct AmqprsProducer {
    #[expect(
        dead_code,
        reason = "we just don't want this to go out of scope, hence they're here"
    )]
    conn: Connection,
    chan: Channel,
}
impl AmqprsProducer {
    pub(crate) async fn send(&self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        tracing::debug!("sending amqp message");

        self.chan
            .basic_publish(
                BasicProperties::default(),
                data.into(),
                BasicPublishArguments::new("", "discord"),
            )
            .await?;

        Ok(())
    }
}

pub(crate) async fn create(addr: &str) -> AmqprsProducer {
    let amqp_addr: OpenConnectionArguments = addr.try_into().expect("couldn't parse amqp uri");

    let amqp_conn = Connection::open(&amqp_addr)
        .await
        .expect("error connecting to amqp");
    amqp_conn
        .register_callback(DefaultConnectionCallback)
        .await
        .expect("failed to register amqp connection callback");

    let amqp_chan = amqp_conn
        .open_channel(None)
        .await
        .expect("couldn't create amqp channel");
    amqp_chan
        .register_callback(DefaultChannelCallback)
        .await
        .expect("failed to register amqp channel callback");
    amqp_chan
        .queue_declare(QueueDeclareArguments::new("discord").durable(true).finish())
        .await
        .expect("error declaring 'discord' amqp queue");

    AmqprsProducer {
        conn: amqp_conn,
        chan: amqp_chan,
    }
}
