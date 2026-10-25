use anyhow::Result;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

/// A window that native dialogs can be owned by, e.g. eframe's `Frame` or
/// `CreationContext`.
///
/// Every dialog should have an owner. On Windows an unowned dialog is an
/// independent top-level window: one click on annelid's main window raises it
/// over the dialog, and because the dialog's modal loop runs on the UI thread,
/// annelid then ignores its close button and right-clicks until the user
/// finds the buried dialog. An owned dialog stays above its owner and disables
/// it while open.
pub trait DialogParent: HasWindowHandle + HasDisplayHandle {}

impl<T: HasWindowHandle + HasDisplayHandle + ?Sized> DialogParent for T {}

/// How an error is described to the user and in the log: the whole chain of
/// context, down to the root cause, e.g. `Failed to load splits file "x.lss":
/// The system cannot find the file specified. (os error 2)`.
pub fn error_report(e: &anyhow::Error) -> String {
    format!("{e:#}")
}

pub fn messagebox_on_error<F>(parent: &dyn DialogParent, f: F)
where
    F: FnOnce() -> Result<()>,
{
    use rfd::{MessageDialog, MessageLevel};
    match f() {
        Ok(()) => {}
        Err(e) => {
            let report = error_report(&e);
            tracing::warn!("about to show messagebox due to: {report}");
            MessageDialog::new()
                .set_parent(parent)
                .set_level(MessageLevel::Error)
                .set_title("Error")
                .set_description(report)
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
            tracing::warn!("{}", error_report(&e));
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
