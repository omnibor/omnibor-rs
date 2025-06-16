//! Functions for initializing logging.

use clap_verbosity_flag::{InfoLevel, Verbosity};
use tracing::Subscriber;
use tracing_subscriber::{
    filter::EnvFilter, layer::SubscriberExt as _, registry::LookupSpan,
    util::SubscriberInitExt as _, Layer,
};

// The environment variable to use when configuring the log.
const LOG_VAR: &str = "OMNIBOR_LOG";

pub fn init_log(verbosity: Verbosity<InfoLevel>, console: bool) {
    let level_filter = adapt_level_filter(verbosity.log_level_filter());
    let filter = EnvFilter::from_env(LOG_VAR).add_directive(level_filter.into());
    let fmt_layer = fmt_layer(filter);
    let registry = tracing_subscriber::registry().with(fmt_layer);

    if console {
        let console_layer = console_subscriber::spawn();
        registry.with(console_layer).init();
    } else {
        registry.init()
    }
}

fn fmt_layer<S>(filter: EnvFilter) -> impl Layer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .event_format(
            tracing_subscriber::fmt::format()
                .with_target(false)
                .compact(),
        )
        .with_filter(filter)
}

/// Convert the clap LevelFilter to the tracing LevelFilter.
fn adapt_level_filter(
    clap_filter: clap_verbosity_flag::LevelFilter,
) -> tracing_subscriber::filter::LevelFilter {
    match clap_filter {
        clap_verbosity_flag::LevelFilter::Off => tracing_subscriber::filter::LevelFilter::OFF,
        clap_verbosity_flag::LevelFilter::Error => tracing_subscriber::filter::LevelFilter::ERROR,
        clap_verbosity_flag::LevelFilter::Warn => tracing_subscriber::filter::LevelFilter::WARN,
        clap_verbosity_flag::LevelFilter::Info => tracing_subscriber::filter::LevelFilter::INFO,
        clap_verbosity_flag::LevelFilter::Debug => tracing_subscriber::filter::LevelFilter::DEBUG,
        clap_verbosity_flag::LevelFilter::Trace => tracing_subscriber::filter::LevelFilter::TRACE,
    }
}
