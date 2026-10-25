use annelid::logging::{sanitize_path, DEFAULT_LEVEL};
use std::path::Path;
use tracing::Level;
use tracing_subscriber::prelude::*;

#[test]
fn default_log_level_keeps_annelids_info_and_quiets_dependencies() {
    // A user's logs couldn't say when annelid started or exited, or when the
    // autosplitter connected: annelid logs those at info, which the default
    // level dropped.
    let filter = tracing_subscriber::EnvFilter::new(DEFAULT_LEVEL.as_str());
    let subscriber = tracing_subscriber::registry().with(filter);
    tracing::subscriber::with_default(subscriber, || {
        assert!(tracing::enabled!(target: "annelid::livesplit_renderer", Level::INFO));
        assert!(!tracing::enabled!(target: "annelid::livesplit_renderer", Level::DEBUG));
        assert!(tracing::enabled!(target: "eframe", Level::WARN));
        assert!(!tracing::enabled!(target: "eframe", Level::INFO));
    });
}

#[test]
fn sanitize_replaces_home_dir() {
    // The home directory should be replaced with ~
    if let Some(dirs) = directories::UserDirs::new() {
        let home = dirs.home_dir();
        let test_path = home.join("Documents").join("splits.lss");
        let result = sanitize_path(&test_path);
        assert!(result.starts_with("~"), "Expected ~ prefix, got: {result}");
        assert!(
            result.contains("Documents"),
            "Expected Documents in path, got: {result}"
        );
        assert!(
            !result.contains(&home.display().to_string()),
            "Home dir should be replaced, got: {result}"
        );
    }
}

#[test]
fn sanitize_leaves_non_home_paths_unchanged() {
    let path = Path::new("/tmp/test/file.txt");
    let result = sanitize_path(path);
    assert_eq!(result, "/tmp/test/file.txt");
}

#[test]
fn sanitize_handles_root_path() {
    let path = Path::new("/");
    let result = sanitize_path(path);
    assert_eq!(result, "/");
}

#[test]
fn sanitize_handles_home_dir_itself() {
    if let Some(dirs) = directories::UserDirs::new() {
        let result = sanitize_path(dirs.home_dir());
        assert_eq!(result, "~");
    }
}
