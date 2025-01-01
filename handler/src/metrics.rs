use std::error::Error;

use metrics_exporter_prometheus::PrometheusBuilder;

pub(crate) fn install() -> Result<(), Box<dyn Error>> {
    // install metrics collector and exporter
    tulpje_shared::metrics::install(
        PrometheusBuilder::new().add_global_label("process", "handler"),
    )?;

    // define metrics
    // ..

    Ok(())
}
