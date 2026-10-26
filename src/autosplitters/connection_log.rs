//! What the SNES polling thread logs about its connection to the usb2snes
//! server (SNI, QUsb2Snes, ...).

use std::time::{Duration, Instant};

/// How often to log a summary.
pub const SUMMARY_INTERVAL: Duration = Duration::from_secs(5 * 60);

/// The most lines to log between summaries, not counting the summaries. This
/// bounds what the polling thread writes, whatever the connection does.
pub const MAX_LINES_PER_SUMMARY: u32 = 50;

/// A line for the log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogLine {
    Info(String),
    Warn(String),
}

impl LogLine {
    pub fn emit(&self) {
        match self {
            LogLine::Info(message) => tracing::info!("{message}"),
            LogLine::Warn(message) => tracing::warn!("{message}"),
        }
    }

    fn push_str(&mut self, s: &str) {
        match self {
            LogLine::Info(message) | LogLine::Warn(message) => message.push_str(s),
        }
    }
}

/// Decides what the SNES polling thread logs about its connection.
///
/// Changes are logged when they happen: connecting, losing the connection, and
/// each new error while it can't connect. While nothing changes, it logs a
/// summary every [`SUMMARY_INTERVAL`] instead of on every poll, so the log
/// shows the thread is still running without repeating the same error every
/// few seconds.
///
/// At most [`MAX_LINES_PER_SUMMARY`] lines are logged between summaries; past
/// that, e.g. when the connection drops right after every reconnect, lines are
/// counted instead and the next summary says how many were left out.
pub struct ConnectionLog {
    state: State,
    /// When the next summary is due.
    summary_due: Instant,
    /// The lines logged since the last summary.
    logged: u32,
    /// The lines left out since the last summary.
    left_out: u32,
}

enum State {
    Disconnected {
        /// When it stopped being connected, or started trying to connect.
        since: Instant,
        /// The last error, if any.
        error: Option<String>,
    },
    Connected {
        server: String,
        since: Instant,
        /// The reads since the last summary.
        reads: Reads,
    },
}

struct Reads {
    since: Instant,
    count: u32,
    total: Duration,
    slowest: Duration,
}

impl Reads {
    fn new(since: Instant) -> Self {
        Reads {
            since,
            count: 0,
            total: Duration::ZERO,
            slowest: Duration::ZERO,
        }
    }

    fn add(&mut self, latency: Duration) {
        self.count += 1;
        self.total += latency;
        self.slowest = self.slowest.max(latency);
    }
}

impl ConnectionLog {
    pub fn new(now: Instant) -> Self {
        ConnectionLog {
            state: State::Disconnected {
                since: now,
                error: None,
            },
            summary_due: now + SUMMARY_INTERVAL,
            logged: 0,
            left_out: 0,
        }
    }

    /// A connection attempt, or a connection, failed with `error`.
    pub fn failed(&mut self, error: String, now: Instant) -> Option<LogLine> {
        let news = match &self.state {
            State::Connected { server, since, .. } => Some(LogLine::Warn(format!(
                "Lost the connection to {server} after {}: {error}",
                format_duration(now - *since)
            ))),
            State::Disconnected {
                error: Some(last), ..
            } if *last == error => None,
            State::Disconnected { .. } => Some(LogLine::Warn(format!("Not connected: {error}"))),
        };
        match &mut self.state {
            State::Connected { .. } => {
                self.state = State::Disconnected {
                    since: now,
                    error: Some(error),
                }
            }
            State::Disconnected { error: last, .. } => *last = Some(error),
        }
        self.log(news, now, None)
    }

    /// Something else the polling thread wants to log happened, e.g. the
    /// autosplitter split.
    pub fn event(&mut self, line: LogLine, now: Instant) -> Option<LogLine> {
        self.log(Some(line), now, None)
    }

    /// A read from `device` on `server` succeeded and took `latency`.
    /// `game_status` describes the game, e.g. `game state 0x08, room 0x91f8`.
    pub fn read(
        &mut self,
        server: &str,
        device: &str,
        latency: Duration,
        now: Instant,
        game_status: impl FnOnce() -> String,
    ) -> Option<LogLine> {
        match &mut self.state {
            State::Disconnected { .. } => {
                let news = LogLine::Info(format!(
                    "Connected to {server}, reading from {device}; first read took {}; {}",
                    format_latency(latency),
                    game_status()
                ));
                self.state = State::Connected {
                    server: server.to_string(),
                    since: now,
                    reads: Reads::new(now),
                };
                self.log(Some(news), now, None)
            }
            State::Connected { reads, .. } => {
                reads.add(latency);
                let game = (now >= self.summary_due).then(game_status);
                self.log(None, now, game)
            }
        }
    }

    /// Logs `news` if it fits under the limit. When a summary is due, logs
    /// `news`, or a summary if there's no news, plus how many lines were left
    /// out since the last summary.
    fn log(
        &mut self,
        news: Option<LogLine>,
        now: Instant,
        game: Option<String>,
    ) -> Option<LogLine> {
        if now < self.summary_due {
            let news = news?;
            if self.logged < MAX_LINES_PER_SUMMARY {
                self.logged += 1;
                return Some(news);
            }
            self.left_out += 1;
            return None;
        }
        let mut line = news.unwrap_or_else(|| self.summary(now, game));
        if self.left_out > 0 {
            line.push_str(&format!(
                " ({} more lines left out since the last summary)",
                self.left_out
            ));
        }
        self.summary_due = now + SUMMARY_INTERVAL;
        self.logged = 0;
        self.left_out = 0;
        if let State::Connected { reads, .. } = &mut self.state {
            *reads = Reads::new(now);
        }
        Some(line)
    }

    /// How things stand.
    fn summary(&self, now: Instant, game: Option<String>) -> LogLine {
        let mut message = match &self.state {
            State::Connected { reads, .. } if reads.count > 0 => format!(
                "{} reads in the last {}, average {}, slowest {}",
                reads.count,
                format_duration(now - reads.since),
                format_latency(reads.total / reads.count),
                format_latency(reads.slowest)
            ),
            State::Connected { server, reads, .. } => format!(
                "Connected to {server}, but no reads in the last {}",
                format_duration(now - reads.since)
            ),
            State::Disconnected { since, error } => {
                let mut message = format!(
                    "Still not connected after {}",
                    format_duration(now - *since)
                );
                if let Some(error) = error {
                    message.push_str(&format!(": {error}"));
                }
                message
            }
        };
        if let Some(game) = game {
            message.push_str(&format!("; {game}"));
        }
        LogLine::Info(message)
    }
}

/// E.g. `8s`, `24m13s`, `1h05m`.
fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    match secs {
        0..60 => format!("{secs}s"),
        60..3600 => format!("{}m{:02}s", secs / 60, secs % 60),
        _ => format!("{}h{:02}m", secs / 3600, secs / 60 % 60),
    }
}

/// E.g. `48.2ms`.
fn format_latency(d: Duration) -> String {
    format!("{:.1}ms", d.as_secs_f64() * 1000.0)
}
