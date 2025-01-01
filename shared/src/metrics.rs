use std::error::Error;

use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_process::Collector as ProcessCollector;

pub fn install(builder: PrometheusBuilder) -> Result<(), Box<dyn Error>> {
    // install recorder and exporter
    builder.install()?;

    // define and start process metrics
    let proc_collector = ProcessCollector::default();
    proc_collector.describe();
    tokio::spawn(async move {
        proc_collector.collect();
    });

    Ok(())
}
