use annelid::utils::from_de_error;

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
