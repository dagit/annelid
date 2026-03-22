use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The log directory, set once at startup.
static LOG_DIR: OnceLock<PathBuf> = OnceLock::new();

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
pub fn init(log_dir: &Path) {
    LOG_DIR.set(log_dir.to_owned()).ok();
    cleanup_old_logs(log_dir, 7);
    init_file_logging(log_dir);
    init_panic_hook();
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

fn init_file_logging(log_dir: &Path) {
    use tracing_appender::rolling;
    use tracing_subscriber::fmt;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::EnvFilter;

    let file_appender = rolling::daily(log_dir, "annelid.log");

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));

    let fmt_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_thread_names(true);

    #[cfg(feature = "tracing")]
    {
        use tracing_chrome::ChromeLayerBuilder;
        let (chrome_layer, _guard) = ChromeLayerBuilder::new().build();
        // Leak the guard so the chrome trace file gets flushed on exit.
        // This is only active when the `tracing` feature is enabled (profiling).
        std::mem::forget(_guard);
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .with(chrome_layer)
            .init();
    }

    #[cfg(not(feature = "tracing"))]
    {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .init();
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
    let now = OffsetDateTime::now_utc();
    // e.g. "2026-03-21 14:30:05 UTC"
    now.format(time::macros::format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second] UTC"
    ))
    .unwrap_or_else(|_| "unknown time".to_string())
}
