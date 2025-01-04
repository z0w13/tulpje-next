use bb8_redis::RedisConnectionManager;
use metrics_exporter_prometheus::PrometheusBuilder;

pub(crate) fn install(
    redis: bb8::Pool<RedisConnectionManager>,
    handler_id: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    // install metrics collector and exporter
    tulpje_shared::metrics::install(
        PrometheusBuilder::new(),
        redis,
        format!("handler-{}", handler_id),
    )?;

    // define metrics
    // ..

    Ok(())
}
