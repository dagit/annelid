use std::ptr;

use gtk::prelude::*;

mod glium_gl_area;
use glium_gl_area::GliumGLArea;

#[cfg(target_os = "linux")]
pub fn main() {
    // Load GL pointers from epoxy (GL context management library used by GTK).
    {
        let library = unsafe { libloading::os::unix::Library::new("libepoxy.so.0") }.unwrap();

        epoxy::load_with(|name| {
            unsafe { library.get::<_>(name.as_bytes()) }
                .map(|symbol| *symbol)
                .unwrap_or(ptr::null())
        });
    }

    let application = gtk::Application::new(
        Some("annelid"),
        Default::default(),
    );
    application.connect_activate(build_ui);
    application.run();
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::new(application);
    window.set_title(Some("Annelid"));

    let widget = GliumGLArea::new();
    window.set_child(Some(&widget));

    window.show();
}
