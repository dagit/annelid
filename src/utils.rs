pub fn messagebox_on_error<F>(f: F)
where
    F: FnOnce() -> std::result::Result<(), Box<dyn std::error::Error>>,
{
    use rfd::{MessageDialog, MessageLevel};
    match f() {
        Ok(()) => {}
        Err(e) => {
            println!("{}", e);
            MessageDialog::new()
                .set_level(MessageLevel::Error)
                .set_title("Error")
                .set_description(format!("{}", e))
                .show();
        }
    }
}

pub fn print_on_error<F>(f: F)
where
    F: FnOnce() -> std::result::Result<(), Box<dyn std::error::Error>>,
{
    match f() {
        Ok(()) => {}
        Err(e) => {
            println!("{}", e);
        }
    }
}

pub fn from_de_error(e: toml::de::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
}
