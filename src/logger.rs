use std::{env, fs, io};
use std::{fs::File, path::Path, sync::Once};

use crate::Result;

use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt, layer::SubscriberExt};
use tracing_subscriber::{fmt::writer::BoxMakeWriter, EnvFilter};
use tracing_subscriber::{util::SubscriberInitExt, Registry};

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

impl Logger {
    fn new(file: Option<File>, env_filter: Option<&str>) -> Result<Logger> {
        let env_filter = env_filter
            .map_or(EnvFilter::default(), EnvFilter::new)
            .add_directive("hyper=info".parse()?)
            .add_directive("mio=info".parse()?)
            .add_directive("h2=info".parse()?)
            .add_directive("tokio=info".parse()?)
            .add_directive("rustls=info".parse()?);

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

        // Disable colors when logging to files.
        let mut event_format = fmt::format().compact().with_ansi(true);
        if should_write_to_file {
            event_format = event_format.with_ansi(false);
        }

        // The format layer is responsible for deciding what the logs look like as they are written.
        let format_layer = fmt::layer().event_format(event_format).with_writer(writer);

        // Finally, the registry tracks tracing spans. We wrap the env layer and format layer and
        // it handles the rest, for the most part.
        Registry::default()
            .with(self.env_filter)
            .with(format_layer)
            .with(ErrorLayer::default())
            .init();
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
