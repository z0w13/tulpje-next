use std::error::Error;

use lapin::{
    options::QueueDeclareOptions, types::FieldTable, Channel, Connection, ConnectionProperties,
};

pub(crate) struct LapinProducer {
    #[expect(
        dead_code,
        reason = "probably needed at some point to handle disconnecting and such"
    )]
    conn: Connection,
    chan: Channel,
}
impl LapinProducer {
    pub(crate) async fn send(&self, data: &[u8]) -> Result<(), Box<dyn Error>> {
        self.chan
            .basic_publish(
                "",
                "discord",
                lapin::options::BasicPublishOptions::default(),
                data,
                lapin::BasicProperties::default(),
            )
            .await?;

        Ok(())
    }
}

pub(crate) async fn create(addr: &str) -> LapinProducer {
    let options = ConnectionProperties::default()
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);
    let conn = Connection::connect(addr, options)
        .await
        .expect("couldn't connect to RabbitMQ");
    let chan = conn
        .create_channel()
        .await
        .expect("couldn't create RabbitMQ channel");
    // declare the queue
    chan.queue_declare(
        "discord",
        QueueDeclareOptions {
            durable: true,
            ..Default::default()
        },
        FieldTable::default(),
    )
    .await
    .expect("couldn't declare queue");

    LapinProducer { conn, chan }
}
