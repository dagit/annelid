mod imp;

use gtk::{
    gdk, glib,
    prelude::{GLAreaExt, WidgetExt},
};

use glib::clone;

use gtk::subclass::prelude::ObjectSubclassIsExt;

use crate::livesplit::LiveSplitCoreRenderer;
use std::cell::RefCell;
use std::rc::Rc;

glib::wrapper! {
    pub struct GliumGLArea(ObjectSubclass<imp::GliumGLArea>)
        @extends gtk::GLArea, gtk::Widget;
}

impl Default for GliumGLArea {
    fn default() -> Self {
        Self::new()
    }
}

impl GliumGLArea {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    pub fn set_core_renderer(&self, core_renderer: Rc<RefCell<LiveSplitCoreRenderer>>) {
        *self.imp().core_renderer.borrow_mut() = Some(core_renderer);
    }

    pub fn set_frame_rate(&mut self, frame_rate: f32) {
        *self.imp().frame_rate.borrow_mut() = frame_rate;
    }

    pub fn use_refresh_timer(&self) {
        use std::time::{Duration, Instant};
        let frame_length = 1000.0 / *self.imp().frame_rate.borrow() as f64;
        let frame_clock = self.frame_clock().expect("frame clock");
        let previous_draw_instant = Rc::new(RefCell::new(Instant::now()));
        #[cfg(debug)]
        println!("frame_length: {}ms", frame_length);
        frame_clock.connect_update(clone!(@weak self as this, @strong previous_draw_instant => move |_clock| {
            if previous_draw_instant.borrow().elapsed() > Duration::from_millis(frame_length.round() as _) {
                #[cfg(debug)]
                println!("{:#?}", previous_draw_instant.borrow().elapsed());
                this.queue_draw();
                *previous_draw_instant.borrow_mut() = Instant::now();
            }
        }));
        frame_clock.begin_updating();
    }
}

unsafe impl glium::backend::Backend for GliumGLArea {
    fn swap_buffers(&self) -> Result<(), glium::SwapBuffersError> {
        // We're supposed to draw (and hence swap buffers) only inside the `render()` vfunc or
        // signal, which means that GLArea will handle buffer swaps for us.
        Ok(())
    }

    unsafe fn get_proc_address(&self, symbol: &str) -> *const std::ffi::c_void {
        epoxy::get_proc_addr(symbol)
    }

    fn get_framebuffer_dimensions(&self) -> (u32, u32) {
        let scale = self.scale_factor();
        let width = self.width();
        let height = self.height();
        ((width * scale) as u32, (height * scale) as u32)
    }

    fn is_current(&self) -> bool {
        match self.context() {
            Some(context) => gdk::GLContext::current() == Some(context),
            None => false,
        }
    }

    unsafe fn make_current(&self) {
        GLAreaExt::make_current(self);
    }
}
