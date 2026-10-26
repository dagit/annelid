//! What the SNES polling thread logs about its usb2snes connection.
//!
//! A user's autosplitter wouldn't split, and their log couldn't say why: of
//! 1328 lines, 1309 were "No devices present", logged on every retry, and
//! nothing said when the autosplitter connected or whether it was reading.

use annelid::autosplitters::connection_log::{
    ConnectionLog, LogLine, MAX_LINES_PER_SUMMARY, SUMMARY_INTERVAL,
};
use std::time::{Duration, Instant};

const SERVER: &str = "SNI-0.0.103";
const DEVICE: &str = "proxy://192.168.1.20:8191?uri=mister";
const RETRY: Duration = Duration::from_secs(3);
const POLL: Duration = Duration::from_millis(16);

fn game() -> String {
    "game state 0x08, room 0x91f8".to_string()
}

fn message(line: &LogLine) -> &str {
    match line {
        LogLine::Info(m) | LogLine::Warn(m) => m,
    }
}

/// Fails with `error` every `RETRY` for `duration`, returning what was logged.
fn keep_failing(
    log: &mut ConnectionLog,
    error: &str,
    now: &mut Instant,
    duration: Duration,
) -> Vec<LogLine> {
    let end = *now + duration;
    let mut lines = vec![];
    while *now < end {
        lines.extend(log.failed(error.to_string(), *now));
        *now += RETRY;
    }
    lines
}

/// Reads every `POLL` for `duration`, returning what was logged.
fn keep_reading(log: &mut ConnectionLog, now: &mut Instant, duration: Duration) -> Vec<LogLine> {
    let end = *now + duration;
    let mut lines = vec![];
    while *now < end {
        lines.extend(log.read(SERVER, DEVICE, Duration::from_millis(5), *now, game));
        *now += POLL;
    }
    lines
}

#[test]
fn a_repeated_error_is_logged_once_then_summarized() {
    let mut now = Instant::now();
    let mut log = ConnectionLog::new(now);
    let lines = keep_failing(
        &mut log,
        "No devices present (server: SNI-0.0.103)",
        &mut now,
        Duration::from_secs(60 * 60),
    );

    let warnings: Vec<_> = lines
        .iter()
        .filter(|l| matches!(l, LogLine::Warn(_)))
        .collect();
    assert_eq!(warnings.len(), 1, "{lines:#?}");
    assert!(message(warnings[0]).contains("No devices present"));
    // An hour of retries is summarized every few minutes, not every retry.
    let summaries = lines.len() - 1;
    let expected = (60 * 60 / SUMMARY_INTERVAL.as_secs()) as usize;
    assert!(
        (expected - 1..=expected).contains(&summaries),
        "expected about {expected} summaries: {lines:#?}"
    );
    assert!(lines[1..]
        .iter()
        .all(|l| matches!(l, LogLine::Info(m) if m.contains("No devices present"))));
}

#[test]
fn a_new_error_is_logged() {
    let now = Instant::now();
    let mut log = ConnectionLog::new(now);
    assert!(log.failed("No devices present".into(), now).is_some());
    assert_eq!(log.failed("No devices present".into(), now + RETRY), None);
    let line = log
        .failed("creating usb2snes connection".into(), now + RETRY * 2)
        .expect("a different error is news");
    assert!(matches!(&line, LogLine::Warn(m) if m.contains("creating usb2snes connection")));
}

#[test]
fn connecting_says_which_server_and_device_and_what_the_game_is_doing() {
    let now = Instant::now();
    let mut log = ConnectionLog::new(now);
    log.failed("No devices present".into(), now);

    let line = log
        .read(SERVER, DEVICE, Duration::from_millis(48), now + RETRY, game)
        .expect("connecting is news");
    let LogLine::Info(m) = &line else {
        panic!("connecting isn't a problem: {line:?}");
    };
    for expected in [SERVER, DEVICE, "48", "room 0x91f8"] {
        assert!(m.contains(expected), "{expected:?} missing from {m:?}");
    }
}

#[test]
fn a_connection_summarizes_its_reads_every_few_minutes() {
    let mut now = Instant::now();
    let mut log = ConnectionLog::new(now);
    let lines = keep_reading(&mut log, &mut now, SUMMARY_INTERVAL * 2 + POLL * 10);

    assert_eq!(lines.len(), 3, "connected, then two summaries: {lines:#?}");
    let reads = SUMMARY_INTERVAL.as_millis() / POLL.as_millis();
    for summary in &lines[1..] {
        let m = message(summary);
        assert!(m.contains(&format!("{reads} reads")), "{m:?}");
        assert!(m.contains("room 0x91f8"), "{m:?}");
    }
}

/// The most lines an hour may log, however the connection behaves: the lines
/// allowed between summaries, plus the summaries.
fn hourly_limit() -> usize {
    let summaries = (60 * 60 / SUMMARY_INTERVAL.as_secs()) as usize + 1;
    summaries * (MAX_LINES_PER_SUMMARY as usize + 1)
}

fn left_out(lines: &[LogLine]) -> bool {
    lines.iter().any(|l| message(l).contains("left out"))
}

#[test]
fn a_connection_that_keeps_dropping_cant_flood_the_log() {
    // Like a MiSTer whose link to SNI drops right after every reconnect.
    let mut now = Instant::now();
    let mut log = ConnectionLog::new(now);
    let end = now + Duration::from_secs(60 * 60);
    let mut lines = vec![];
    while now < end {
        lines.extend(log.read(SERVER, DEVICE, Duration::from_millis(5), now, game));
        now += Duration::from_millis(100);
        lines.extend(log.failed("GetAddress failed".into(), now));
        now += Duration::from_secs(1);
    }
    assert!(
        lines.len() <= hourly_limit(),
        "{} lines in an hour, limit {}",
        lines.len(),
        hourly_limit()
    );
    assert!(left_out(&lines), "should say lines were left out");
}

#[test]
fn errors_that_keep_changing_cant_flood_the_log() {
    let mut now = Instant::now();
    let mut log = ConnectionLog::new(now);
    let end = now + Duration::from_secs(60 * 60);
    let mut lines = vec![];
    for error in ["reset without closing handshake", "connection refused"]
        .iter()
        .cycle()
    {
        if now >= end {
            break;
        }
        lines.extend(log.failed(error.to_string(), now));
        now += Duration::from_secs(1);
    }
    assert!(
        lines.len() <= hourly_limit(),
        "{} lines in an hour, limit {}",
        lines.len(),
        hourly_limit()
    );
    assert!(left_out(&lines), "should say lines were left out");
}

#[test]
fn autosplitter_events_cant_flood_the_log() {
    // e.g. garbage reads that look like a split on every poll.
    let mut now = Instant::now();
    let mut log = ConnectionLog::new(now);
    let end = now + Duration::from_secs(60 * 60);
    let mut lines = vec![];
    while now < end {
        lines.extend(log.read(SERVER, DEVICE, Duration::from_millis(5), now, game));
        lines.extend(log.event(LogLine::Info("Autosplitter split".into()), now));
        now += Duration::from_millis(50);
    }
    assert!(
        lines.len() <= hourly_limit(),
        "{} lines in an hour, limit {}",
        lines.len(),
        hourly_limit()
    );
    assert!(left_out(&lines), "should say lines were left out");
    // The summaries keep coming, so the log still shows it's reading.
    let summaries = lines
        .iter()
        .filter(|l| message(l).contains(" reads in the last "))
        .count();
    assert!(summaries >= 11, "{summaries} summaries");
}

#[test]
fn losing_the_connection_is_logged_even_if_the_error_is_the_same() {
    let mut now = Instant::now();
    let mut log = ConnectionLog::new(now);
    let reset = "WebSocket protocol error: Connection reset without closing handshake";
    log.failed(reset.into(), now);
    keep_reading(&mut log, &mut now, Duration::from_secs(25 * 60));

    let line = log
        .failed(reset.into(), now)
        .expect("losing the connection is news");
    let LogLine::Warn(m) = &line else {
        panic!("losing the connection is a problem: {line:?}");
    };
    assert!(m.contains("Lost"), "{m:?}");
    assert!(
        m.contains(SERVER),
        "should say what it was connected to: {m:?}"
    );
    assert!(
        m.contains("25m"),
        "should say how long it was connected: {m:?}"
    );
    assert!(m.contains(reset), "{m:?}");
}
