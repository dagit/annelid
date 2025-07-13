use anyhow::Result;

pub fn messagebox_on_error<F>(f: F)
where
    F: FnOnce() -> Result<()>,
{
    use rfd::{MessageDialog, MessageLevel};
    match f() {
        Ok(()) => {}
        Err(e) => {
            println!("about to show messagebox due to: {e}");
            MessageDialog::new()
                .set_level(MessageLevel::Error)
                .set_title("Error")
                .set_description(format!("{e}"))
                .show();
        }
    }
}

pub fn print_on_error<F>(f: F)
where
    F: FnOnce() -> Result<()>,
{
    match f() {
        Ok(()) => {}
        Err(e) => {
            println!("{}", e);
        }
    }
}

pub fn queue_on_error<F>(queue: &mut Vec<anyhow::Error>, f: F)
where
    F: FnOnce() -> Result<()>,
{
    match f() {
        Ok(()) => {}
        Err(e) => {
            queue.push(e);
        }
    }
}

pub fn from_de_error(e: toml::de::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
}
