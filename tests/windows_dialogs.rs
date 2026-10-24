//! End-to-end checks of annelid's native dialogs on Windows.
//!
//! These launch the real binary and inspect its windows through Win32, so they
//! need a desktop session. annelid is killed when each test ends rather than
//! closed, so it never writes settings.toml.
#![cfg(windows)]

use std::process::{Child, Command};
use std::time::{Duration, Instant};
use windows_sys::core::BOOL;
use windows_sys::Win32::Foundation::{HWND, LPARAM, TRUE};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetWindow, GetWindowTextW, GetWindowThreadProcessId,
    IsWindowVisible, GW_OWNER,
};

#[derive(Debug, Clone)]
struct TopLevelWindow {
    hwnd: HWND,
    class: String,
    title: String,
    owner: HWND,
    enabled: bool,
}

/// Visible top-level windows belonging to process `pid`.
fn windows_of(pid: u32) -> Vec<TopLevelWindow> {
    unsafe extern "system" fn collect(hwnd: HWND, lparam: LPARAM) -> BOOL {
        // SAFETY: `lparam` is the `&mut Vec<HWND>` passed to EnumWindows below,
        // which outlives the enumeration.
        let found = unsafe { &mut *(lparam as *mut Vec<HWND>) };
        found.push(hwnd);
        TRUE
    }
    let mut all: Vec<HWND> = Vec::new();
    // SAFETY: `collect` only touches the Vec behind `lparam`.
    unsafe { EnumWindows(Some(collect), &mut all as *mut Vec<HWND> as LPARAM) };

    let text = |buf: &[u16], len: i32| String::from_utf16_lossy(&buf[..len.max(0) as usize]);
    all.into_iter()
        .filter_map(|hwnd| {
            // SAFETY: plain queries on window handles from EnumWindows; a handle
            // that has since been destroyed just makes these calls fail.
            unsafe {
                let mut owner_pid = 0;
                GetWindowThreadProcessId(hwnd, &mut owner_pid);
                if owner_pid != pid || IsWindowVisible(hwnd) == 0 {
                    return None;
                }
                let mut class = [0u16; 256];
                let class_len = GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32);
                let mut title = [0u16; 256];
                let title_len = GetWindowTextW(hwnd, title.as_mut_ptr(), title.len() as i32);
                Some(TopLevelWindow {
                    hwnd,
                    class: text(&class, class_len),
                    title: text(&title, title_len),
                    owner: GetWindow(hwnd, GW_OWNER),
                    enabled: IsWindowEnabled(hwnd) != 0,
                })
            }
        })
        .collect()
}

/// Kills annelid when the test ends, whether it passed or not.
struct Annelid(Child);

impl Drop for Annelid {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn startup_error_dialog_is_owned_by_the_main_window() {
    // A splits file that can't be loaded makes annelid show an error box at
    // startup — what the user who reported this saw on every launch.
    let missing = std::env::temp_dir().join("annelid-test-missing-splits.lss");
    let _ = std::fs::remove_file(&missing);
    let app = Annelid(
        Command::new(env!("CARGO_BIN_EXE_annelid"))
            .arg("--load-splits")
            .arg(&missing)
            .args(["--use-autosplitter", "no", "--global-hotkeys", "no"])
            .spawn()
            .expect("failed to launch annelid"),
    );
    let pid = app.0.id();

    let deadline = Instant::now() + Duration::from_secs(60);
    let (main, dialog) = loop {
        let windows = windows_of(pid);
        let main = windows.iter().find(|w| w.title == "Annelid");
        let dialog = windows
            .iter()
            .find(|w| w.class == "#32770" && w.title == "Error");
        if let (Some(main), Some(dialog)) = (main, dialog) {
            break (main.clone(), dialog.clone());
        }
        assert!(
            Instant::now() < deadline,
            "annelid never showed its startup error; windows: {windows:?}"
        );
        std::thread::sleep(Duration::from_millis(250));
    };

    // An unowned dialog is an independent top-level window: one click on the
    // main window raises it over the dialog, and since the dialog's modal loop
    // holds the UI thread, the timer stops responding — closing it and
    // right-clicking for the control panel do nothing — until the user finds
    // the dialog again.
    assert_eq!(
        dialog.owner, main.hwnd,
        "the error dialog should be owned by the main window; dialog: {dialog:?}, main: {main:?}"
    );
    assert!(
        !main.enabled,
        "the main window should be disabled while its dialog is open; main: {main:?}"
    );
}
