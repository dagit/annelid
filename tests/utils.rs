use annelid::utils::{error_report, from_de_error};

#[test]
fn error_report_says_why_loading_failed() {
    // A user's splits failed to load on every launch, and neither the error
    // dialog nor the log said why: both showed only the outermost context.
    use anyhow::Context;
    let path = std::env::temp_dir().join("annelid-test-error-report-missing.lss");
    let _ = std::fs::remove_file(&path);
    let err = std::fs::File::open(&path)
        .with_context(|| format!("Failed to load splits file {path:?}"))
        .unwrap_err();

    let report = error_report(&err);
    assert!(report.starts_with("Failed to load splits file"), "{report}");
    let cause = err.root_cause().to_string();
    assert!(
        report.contains(&cause),
        "the report should include the cause {cause:?}, got {report:?}"
    );
}

#[test]
fn error_report_does_not_repeat_a_cause_the_message_already_includes() {
    // tungstenite's errors put their cause in their own message, so a user's
    // log said "... (os error 10054): ... (os error 10054)".
    use anyhow::Context;
    let reset = std::io::Error::new(
        std::io::ErrorKind::ConnectionReset,
        "An existing connection was forcibly closed by the remote host. (os error 10054)",
    );
    let err = anyhow::Error::from(tungstenite::Error::Io(reset));
    assert_eq!(
        error_report(&err),
        "IO error: An existing connection was forcibly closed by the remote host. (os error 10054)"
    );

    let err = Err::<(), _>(tungstenite::Error::Url(
        tungstenite::error::UrlError::UnableToConnect("ws://localhost:23074/".into()),
    ))
    .context("creating usb2snes connection")
    .unwrap_err();
    assert_eq!(
        error_report(&err),
        "creating usb2snes connection: URL error: Unable to connect to ws://localhost:23074/"
    );
}

#[test]
fn from_de_error_produces_invalid_data() {
    let toml_err = toml::from_str::<toml::Value>("{{{{bad").unwrap_err();
    let io_err = from_de_error(toml_err);
    assert_eq!(io_err.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn from_de_error_preserves_message() {
    let toml_err = toml::from_str::<toml::Value>("not = [valid").unwrap_err();
    let msg = toml_err.to_string();
    let io_err = from_de_error(toml_err);
    assert!(
        io_err.to_string().contains(&msg),
        "Error message should contain original: got '{}'",
        io_err
    );
}
