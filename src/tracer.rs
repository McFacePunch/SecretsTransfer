use std::sync::Arc;
use std::fs::File;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::fmt;
use tracing_subscriber::filter;

pub fn init_tracing(debug_log_path: &str) {
    let stdout_log = fmt::layer().pretty();

    // Create or open the log file specified in the config.
    let file = File::create(debug_log_path)
        .unwrap_or_else(|_| panic!("Error creating debug log file: {:?}", debug_log_path));

    let debug_log = fmt::layer().with_writer(Arc::new(file));

    // A layer that collects metrics using specific events.
    let metrics_layer = filter::LevelFilter::INFO;

    tracing_subscriber::registry()
        .with(
            stdout_log
                .with_filter(filter::LevelFilter::INFO)
                .and_then(debug_log)
                .with_filter(filter::filter_fn(|metadata| {
                    !metadata.target().starts_with("metrics")
                }))
        )
        .with(
            metrics_layer.with_filter(filter::filter_fn(|metadata| {
                metadata.target().starts_with("metrics")
            }))
        )
        .init();
}
