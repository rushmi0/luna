use tracing::Level;

/// Log verbosity, exposed to every platform via UniFFI.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for Level {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Error => Self::ERROR,
            LogLevel::Warn  => Self::WARN,
            LogLevel::Info  => Self::INFO,
            LogLevel::Debug => Self::DEBUG,
            LogLevel::Trace => Self::TRACE,
        }
    }
}

// ── Desktop (Linux / macOS / Windows) ────────────────────────────────────────

#[uniffi::export]
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub fn init_logger(level: LogLevel) {
    let level: Level = level.into();
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(level)
        .finish();
    match tracing::subscriber::set_global_default(subscriber) {
        Ok(_)  => tracing::info!("Luna {} — logger initialized", env!("CARGO_PKG_VERSION")),
        Err(e) => eprintln!("Luna: failed to initialize logger: {e}"),
    }
}

// ── Mobile (Android + iOS) ────────────────────────────────────────────────────

#[uniffi::export]
#[cfg(any(target_os = "android", target_os = "ios"))]
pub fn init_logger(level: LogLevel) {
    use tracing_subscriber::{filter::Targets, fmt, layer::SubscriberExt, util::SubscriberInitExt, Layer};

    let level: Level = level.into();

    // Android → Logcat via paranoid_android; iOS → plain fmt layer without ANSI
    #[cfg(target_os = "android")]
    let layer = fmt::layer()
        .with_writer(paranoid_android::AndroidLogMakeWriter::new("luna".to_owned()));

    #[cfg(not(target_os = "android"))]
    let layer = fmt::layer();

    let targets = Targets::new().with_default(level);

    let res = tracing_subscriber::registry()
        .with(layer.with_ansi(false).with_file(false).with_filter(targets))
        .try_init();

    match res {
        Ok(_)  => tracing::info!("Luna {} — logger initialized", env!("CARGO_PKG_VERSION")),
        Err(e) => eprintln!("Luna: failed to initialize logger: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::Level;

    #[test]
    fn error_maps_to_tracing_error() {
        assert_eq!(Level::from(LogLevel::Error), Level::ERROR);
    }

    #[test]
    fn warn_maps_to_tracing_warn() {
        assert_eq!(Level::from(LogLevel::Warn), Level::WARN);
    }

    #[test]
    fn info_maps_to_tracing_info() {
        assert_eq!(Level::from(LogLevel::Info), Level::INFO);
    }

    #[test]
    fn debug_maps_to_tracing_debug() {
        assert_eq!(Level::from(LogLevel::Debug), Level::DEBUG);
    }

    #[test]
    fn trace_maps_to_tracing_trace() {
        assert_eq!(Level::from(LogLevel::Trace), Level::TRACE);
    }
}