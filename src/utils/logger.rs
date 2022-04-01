use std::{env, fs, io};
use std::{fs::File, path::Path, sync::Once};

use color_eyre::Result;
use time::format_description;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    fmt::{self, writer::BoxMakeWriter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Registry,
};

static INIT: Once = Once::new();

/// Installs logger and error reporter, using `RUST_LOG` and `RUST_BACKTRACE` as filters.
///
/// Optionally, pass in a `log_file` to redirect log output away from `stdout`.
pub fn install(log_file: Option<&Path>) {
    INIT.call_once(|| {
        install_logger(log_file).unwrap();
        color_eyre::config::HookBuilder::default()
            .display_env_section(false)
            .install()
            .unwrap();
    });
}

fn install_logger(log_file: Option<&Path>) -> Result<()> {
    if env::var("RUST_BACKTRACE").is_err() {
        env::set_var("RUST_BACKTRACE", "short");
    }

    let log_file = match log_file {
        Some(path) => {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            Some(
                fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(path)?,
            )
        }
        None => None,
    };

    let filter = env::var("RUST_LOG").ok();
    Logger::new(log_file, filter.as_deref())?.install()?;

    Ok(())
}

/// A simple logger.
///
/// Logs are redirected to a `log_file` if one is provided.
#[derive(Debug)]
struct Logger {
    env_filter: EnvFilter,
    file: Option<File>,
}

// Uncomment out tokio stuff to enable the tokio console.
impl Logger {
    fn new(file: Option<File>, env_filter: Option<&str>) -> Result<Logger> {
        let env_filter = env_filter.map_or(EnvFilter::default(), EnvFilter::new);
        // .add_directive("tokio=trace".parse()?)
        // .add_directive("runtime=trace".parse()?);

        Ok(Logger { env_filter, file })
    }

    fn install(self) -> Result<()> {
        // We have to box the writer to allow the internal writer type to be generic,
        // i.e. dynamic dispatch. Do we need an Arc around the file?
        let should_write_to_file = self.file.is_some();
        let writer = match self.file {
            Some(file) => BoxMakeWriter::new(file),
            None => BoxMakeWriter::new(io::stdout),
        };

        let environment = env::var("ENVIRONMENT")?;

        // Format time and logs into the format_layer for display.

        let time_format = "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:2]";
        let time_format = format_description::parse(time_format)?;
        let timer = fmt::time::UtcTime::new(time_format);

        let mut prod_fmt = fmt::format().with_timer(timer.clone());
        let mut local_fmt = fmt::format().pretty().with_timer(timer);

        // Disable colors when logging to files.
        if should_write_to_file {
            local_fmt = local_fmt.with_ansi(false);
            prod_fmt = prod_fmt.with_ansi(false);
        }

        // let console_layer = console_subscriber::ConsoleLayer::builder().spawn();

        // Finally, the registry tracks tracing spans. We wrap the env layer and format layer and
        // it handles the rest, for the most part.
        let registry = Registry::default()
            .with(self.env_filter)
            // .with(console_layer)
            .with(ErrorLayer::default());

        if environment == "local" {
            let local_fmt = fmt::layer().event_format(local_fmt).with_writer(writer);
            registry.with(local_fmt).init();
        } else {
            let format_layer = fmt::layer().event_format(prod_fmt).with_writer(writer);
            registry.with(format_layer).init();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs, path::Path};
    use tracing::{debug, error, info, trace, warn};

    #[test]
    fn test_log() -> Result<()> {
        env::set_var("RUST_LOG", "trace");
        let path = Path::new("/tmp/protols/test/log_test.log");
        install(Some(path));

        info!("info");
        warn!("warn");
        debug!("debug");
        error!("error");
        trace!("trace");

        let res = fs::read_to_string(path)?;
        let expected = ["info", "warn", "debug", "error", "trace"];
        for (line, &expect) in res.lines().zip(expected.iter()) {
            assert!(line.contains(&expect.to_ascii_uppercase()));
            assert!(line.contains(expect));
        }

        fs::remove_file(path)?;
        Ok(())
    }
}
