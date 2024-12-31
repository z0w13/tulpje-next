use futures::StreamExt;
use lapin::{
    options::{BasicConsumeOptions, QueueDeclareOptions},
    types::FieldTable,
    Connection, ConnectionProperties,
};
use tokio::sync::mpsc;

pub(crate) struct LapinConsumer {
    queue: mpsc::UnboundedReceiver<Vec<u8>>,
}
impl LapinConsumer {
    pub(crate) async fn recv(&mut self) -> Option<Vec<u8>> {
        self.queue.recv().await
    }
}

pub(crate) async fn create(addr: &str) -> LapinConsumer {
    let rabbitmq_options = ConnectionProperties::default()
        .with_executor(tokio_executor_trait::Tokio::current())
        .with_reactor(tokio_reactor_trait::Tokio);
    let rabbitmq_conn = Connection::connect(addr, rabbitmq_options)
        .await
        .expect("couldn't connect to RabbitMQ");
    let rabbitmq_chan = rabbitmq_conn
        .create_channel()
        .await
        .expect("couldn't create RabbitMQ channel");
    // declare the queue
    rabbitmq_chan
        .queue_declare(
            "discord",
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .expect("couldn't declare queue");
    let mut rabbitmq_consumer = rabbitmq_chan
        .basic_consume(
            "discord",
            "handler",
            BasicConsumeOptions {
                no_ack: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .expect("couldn't create consumer");

    let (message_queue_send, message_queue_recv) = mpsc::unbounded_channel::<Vec<u8>>();
    tokio::spawn(async move {
        loop {
            let message = match rabbitmq_consumer.next().await {
                Some(Ok(message)) => message.data,
                Some(Err(err)) => {
                    tracing::error!("error receiving message: {}", err);
                    continue;
                }
                None => break,
            };

            if let Err(err) = message_queue_send.send(message) {
                tracing::error!("error putting message on queue: {}", err);
            }
        }
    });

    LapinConsumer {
        queue: message_queue_recv,
    }
}
