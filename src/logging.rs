use parking_lot::Mutex;
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use tracing_subscriber::EnvFilter;

/// The log directory, set once at startup.
static LOG_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Handle for reloading the log filter at runtime.
static FILTER_HANDLE: OnceLock<
    tracing_subscriber::reload::Handle<EnvFilter, tracing_subscriber::Registry>,
> = OnceLock::new();

/// Maximum number of log lines kept in the in-memory ring buffer.
const MAX_UI_LOG_LINES: usize = 500;

/// Shared ring buffer of formatted log lines for the UI viewer.
pub type LogBuffer = Arc<Mutex<VecDeque<String>>>;

/// Log level presets available in the UI.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

impl LogLevel {
    pub const ALL: &[LogLevel] = &[
        LogLevel::Error,
        LogLevel::Warn,
        LogLevel::Info,
        LogLevel::Debug,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            LogLevel::Error => "error",
            LogLevel::Warn => "warn",
            LogLevel::Info => "info",
            LogLevel::Debug => "annelid=debug,warn",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            LogLevel::Error => "Error",
            LogLevel::Warn => "Warn (default)",
            LogLevel::Info => "Info",
            LogLevel::Debug => "Debug",
        }
    }
}

/// Change the active log filter at runtime.
pub fn set_log_level(level: LogLevel) {
    if let Some(handle) = FILTER_HANDLE.get() {
        match EnvFilter::try_new(level.as_str()) {
            Ok(new_filter) => {
                if handle.reload(new_filter).is_err() {
                    eprintln!("Failed to reload log filter");
                }
            }
            Err(e) => eprintln!("Invalid log filter: {e}"),
        }
    }
}

/// Sanitizes a path by replacing the user's home directory with `~`.
pub fn sanitize_path(p: &Path) -> String {
    let s = p.display().to_string();
    match directories::UserDirs::new() {
        Some(dirs) => s.replace(&dirs.home_dir().display().to_string(), "~"),
        None => s,
    }
}

/// Initializes file logging and the panic hook.
///
/// Call this early in `main()`, after determining the project directory.
/// - Sets up a daily-rotating log file (`annelid.log`) in `log_dir`.
/// - Installs a panic hook that writes crash info to `annelid-crash.log`
///   and shows an error dialog via rfd.
pub fn init(log_dir: &Path) -> LogBuffer {
    LOG_DIR.set(log_dir.to_owned()).ok();
    cleanup_old_logs(log_dir, 7);
    let log_buffer = Arc::new(Mutex::new(VecDeque::with_capacity(MAX_UI_LOG_LINES)));
    init_file_logging(log_dir, log_buffer.clone());
    init_panic_hook();
    log_buffer
}

/// Returns the log directory path, if initialized.
pub fn log_dir() -> Option<&'static Path> {
    LOG_DIR.get().map(|p| p.as_path())
}

/// Opens the log directory in the platform's file explorer.
pub fn open_log_dir() {
    let Some(dir) = LOG_DIR.get() else {
        return;
    };
    #[cfg(target_os = "linux")]
    let cmd = "xdg-open";
    #[cfg(target_os = "macos")]
    let cmd = "open";
    #[cfg(target_os = "windows")]
    let cmd = "explorer";
    let _ = std::process::Command::new(cmd).arg(dir).spawn();
}

/// Deletes annelid log files older than `max_age_days` days.
/// Only touches files matching `annelid.log.*` and `annelid-crash.log`.
/// The crash log is truncated (not deleted) if older than the threshold,
/// so users always have the most recent crash info.
fn cleanup_old_logs(log_dir: &Path, max_age_days: u64) {
    let max_age = std::time::Duration::from_secs(max_age_days * 24 * 60 * 60);
    let Ok(entries) = std::fs::read_dir(log_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name_str) = name.to_str() else {
            continue;
        };
        // Only touch our own log files
        if !name_str.starts_with("annelid.log.") {
            continue;
        }
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        let Ok(age) = std::time::SystemTime::now().duration_since(modified) else {
            continue;
        };
        if age > max_age {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

fn init_file_logging(log_dir: &Path, log_buffer: LogBuffer) {
    use tracing_appender::rolling;
    use tracing_subscriber::fmt;
    use tracing_subscriber::prelude::*;

    let file_appender = rolling::daily(log_dir, "annelid.log");

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));
    let (filter_layer, reload_handle) = tracing_subscriber::reload::Layer::new(filter);
    FILTER_HANDLE.set(reload_handle).ok();

    let fmt_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_thread_names(true);

    let ui_layer = UiLogLayer { buffer: log_buffer };

    #[cfg(feature = "tracing")]
    {
        use tracing_chrome::ChromeLayerBuilder;
        let (chrome_layer, _guard) = ChromeLayerBuilder::new().build();
        // Leak the guard so the chrome trace file gets flushed on exit.
        // This is only active when the `tracing` feature is enabled (profiling).
        std::mem::forget(_guard);
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .with(ui_layer)
            .with(chrome_layer)
            .init();
    }

    #[cfg(not(feature = "tracing"))]
    {
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(fmt_layer)
            .with(ui_layer)
            .init();
    }
}

/// A tracing layer that captures formatted log lines into a shared ring buffer.
struct UiLogLayer {
    buffer: LogBuffer,
}

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for UiLogLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        use std::fmt::Write;

        let metadata = event.metadata();
        let mut message = String::new();
        let _ = write!(
            message,
            "{} [{}] {}: ",
            time_stamp(),
            metadata.level(),
            metadata.target()
        );

        // Extract the message field from the event
        struct MessageVisitor<'a>(&'a mut String);
        impl tracing::field::Visit for MessageVisitor<'_> {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                use std::fmt::Write;
                if field.name() == "message" {
                    let _ = write!(self.0, "{:?}", value);
                } else {
                    let _ = write!(self.0, " {}={:?}", field.name(), value);
                }
            }
            fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
                use std::fmt::Write;
                if field.name() == "message" {
                    let _ = write!(self.0, "{}", value);
                } else {
                    let _ = write!(self.0, " {}={}", field.name(), value);
                }
            }
        }

        event.record(&mut MessageVisitor(&mut message));

        let mut buf = self.buffer.lock();
        if buf.len() >= MAX_UI_LOG_LINES {
            buf.pop_front();
        }
        buf.push_back(message);
    }
}

fn init_panic_hook() {
    let default_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        // 1. Run the default hook (prints to stderr)
        default_hook(info);

        // 2. Build crash report
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic payload".to_string()
        };

        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("unnamed");

        let backtrace = std::backtrace::Backtrace::force_capture();

        let report = format!(
            "Annelid crash report\n\
             Time: {}\n\
             Thread: {}\n\
             Location: {}\n\
             Message: {}\n\n\
             Backtrace:\n{}",
            time_stamp(),
            thread_name,
            location,
            payload,
            backtrace,
        );

        // 3. Write to crash file
        if let Some(log_dir) = LOG_DIR.get() {
            let crash_path = log_dir.join("annelid-crash.log");
            // Append so we keep history of crashes
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&crash_path)
            {
                use std::io::Write;
                let _ = writeln!(f, "{}\n{}\n", "=".repeat(60), report);
            }
        }

        // 4. Show dialog (best-effort — may fail if no display)
        let short_msg = format!(
            "Annelid has crashed.\n\n\
             {payload}\n\
             at {location}\n\n\
             A crash log has been saved. Please include it when reporting this bug."
        );

        // rfd may panic or fail if the event loop is dead, so wrap in catch_unwind
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Error)
                .set_title("Annelid Crash")
                .set_description(&short_msg)
                .show();
        }));
    }));
}

fn time_stamp() -> String {
    use time::OffsetDateTime;
    // Try local time; fall back to UTC if the platform doesn't support it
    // (e.g. some multi-threaded Unix environments).
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    let is_utc = now.offset().is_utc();
    let formatted = now
        .format(time::macros::format_description!(
            "[year]-[month]-[day] [hour]:[minute]:[second]"
        ))
        .unwrap_or_else(|_| "unknown time".to_string());
    if is_utc {
        format!("{formatted} UTC")
    } else {
        formatted
    }
}
