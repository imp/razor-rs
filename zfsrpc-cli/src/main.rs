#![cfg_attr(feature = "pedantic", warn(clippy::pedantic))]
#![warn(clippy::use_self)]
#![warn(clippy::map_flatten)]
#![warn(clippy::map_unwrap_or)]
#![warn(deprecated_in_future)]
#![warn(future_incompatible)]
#![warn(noop_method_call)]
#![warn(unreachable_pub)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2021_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(unused)]
#![deny(warnings)]

use tracing::error;
use tracing_subscriber::{fmt, EnvFilter};

pub mod cli;

const DEFAULT_TRACE_LEVEL: &str = "info";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_TRACE_LEVEL));

    fmt()
        .with_env_filter(filter)
        .pretty()
        .with_timer(fmt::time::UtcTime::rfc_3339())
        .init();

    if let Err(e) = cli::Cli::execute().await {
        error!(?e);
    }

    Ok(())
}
