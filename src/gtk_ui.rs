use std::ptr;

use gtk::prelude::*;

mod glium_gl_area;

use crate::livesplit;

use glium_gl_area::GliumGLArea;

use crate::livesplit::LiveSplitCoreRenderer;
use parking_lot::RwLock;
use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::sync::Arc;

pub fn main(
    core_renderer: LiveSplitCoreRenderer,
    frame_rate: f32,
    _polling_rate: f32,
    _latency: Arc<RwLock<(f32, f32)>>,
    _sync_receiver: std::sync::mpsc::Receiver<livesplit::ThreadEvent>,
) -> std::result::Result<(), Box<dyn Error>> {
    // Load GL pointers from epoxy (GL context management library used by GTK).
    {
        #[cfg(target_os = "macos")]
        let library = unsafe { libloading::os::unix::Library::new("libepoxy.0.dylib") }.unwrap();
        #[cfg(all(unix, not(target_os = "macos")))]
        let library = unsafe { libloading::os::unix::Library::new("libepoxy.so.0") }.unwrap();
        #[cfg(windows)]
        let library = libloading::os::windows::Library::open_already_loaded("libepoxy-0.dll")
            .or_else(|_| libloading::os::windows::Library::open_already_loaded("epoxy-0.dll"))
            .unwrap();

        epoxy::load_with(|name| {
            unsafe { library.get::<_>(name.as_bytes()) }
                .map(|symbol| *symbol)
                .unwrap_or(ptr::null())
        });
    }
    let core_renderer = Rc::new(RefCell::new(core_renderer));
    core_renderer
        .borrow_mut()
        .timer
        .write()
        .expect("to take the lock")
        .start();

    let application = gtk::Application::new(Some("app.annelid"), Default::default());
    application.connect_activate(move |application| {
        build_ui(core_renderer.clone(), frame_rate, application)
    });
    application.run_with_args(&[] as &[&str]);
    Ok(())
}

fn build_ui(
    core_renderer: Rc<RefCell<LiveSplitCoreRenderer>>,
    frame_rate: f32,
    application: &gtk::Application,
) {
    let window = gtk::ApplicationWindow::new(application);
    window.set_title(Some("Annelid"));

    let mut widget = GliumGLArea::new();
    widget.set_core_renderer(core_renderer);
    widget.set_frame_rate(frame_rate);
    window.set_child(Some(&widget));

    window.show();
}
