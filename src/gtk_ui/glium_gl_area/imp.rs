use crate::livesplit::LiveSplitCoreRenderer;
use glium::{
    implement_vertex, index::PrimitiveType, program, uniform, Frame, IndexBuffer, Surface,
    VertexBuffer,
};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

#[derive(Copy, Clone)]
struct Vertex {
    a_pos: [f32; 2],
    a_tc: [f32; 2],
}

implement_vertex!(Vertex, a_pos, a_tc);

pub struct Renderer {
    context: Rc<glium::backend::Context>,
    index_buffer: IndexBuffer<u16>,
    program: glium::Program,
    frame_buffer: RefCell<Vec<u8>>,
}

impl Renderer {
    fn new(context: Rc<glium::backend::Context>) -> Self {
        println!("Version: {}", context.get_opengl_version_string());
        println!("Vender: {}", context.get_opengl_vendor_string());
        println!("Renderer: {}", context.get_opengl_renderer_string());

        let index_buffer = IndexBuffer::new(
            &context,
            PrimitiveType::TrianglesList,
            &[0u16, 1, 3, 1, 2, 3],
        )
        .expect("Create index buffer");
        let program = program!(&context,
            140 => {

                vertex: "
                    #version 140
                    
                    uniform   vec2 u_screen_size;
                    attribute vec2 a_pos;
                    attribute vec2 a_tc;
                    varying   vec2 v_tc;
                    
                    void main() {
                        gl_Position = vec4(
                                          2.0 * a_pos.x / u_screen_size.x - 1.0,
                                          1.0 - 2.0 * a_pos.y / u_screen_size.y,
                                          0.0,
                                          1.0);
                        v_tc = a_tc;
                    }
                ",
                fragment: "
                    #version 140
                    
                    uniform sampler2D u_sampler;
                    
                    varying vec2      v_tc;
                    
                    void main() {
                        gl_FragColor = texture2D(u_sampler, v_tc);
                    }
                "
            },
            330 => {

                vertex: "
                    #version 330
                    
                    uniform vec2 u_screen_size;
                    in      vec2 a_pos;
                    in      vec2 a_tc;
                    out     vec2 v_tc;
                    
                    void main() {
                        gl_Position = vec4(
                                          2.0 * a_pos.x / u_screen_size.x - 1.0,
                                          1.0 - 2.0 * a_pos.y / u_screen_size.y,
                                          0.0,
                                          1.0);
                        v_tc = a_tc;
                    }
                ",
                fragment: "
                    #version 330
                    
                    uniform sampler2D u_sampler;
                    
                    in      vec2      v_tc;
                    
                    out     vec4      fragmentColor;
                    
                    void main() {
                        fragmentColor = texture(u_sampler, v_tc);
                    }
                "
            },
        )
        .expect("Create gl program");

        Renderer {
            context,
            index_buffer,
            program,
            frame_buffer: RefCell::new(vec![0; 0]),
        }
    }

    fn draw(&self, core_renderer: &mut LiveSplitCoreRenderer) {
        use glium::texture::ToClientFormat;
        use std::borrow::Cow;

        let mut frame = Frame::new(
            self.context.clone(),
            self.context.get_framebuffer_dimensions(),
        );

        let timer = core_renderer.timer.read().expect("to take lock");
        let snapshot = timer.snapshot();
        match &mut core_renderer.layout_state {
            None => {
                core_renderer.layout_state = Some(core_renderer.layout.state(&snapshot));
            }
            Some(layout_state) => {
                core_renderer.layout.update_state(layout_state, &snapshot);
            }
        };

        if let Some(layout_state) = &core_renderer.layout_state {
            let (w, h) = self.context.get_framebuffer_dimensions();
            let szu32 = [w as u32, h as u32];
            let mut frame_buffer = self.frame_buffer.borrow_mut();
            frame_buffer.resize(w as usize * h as usize * 4, 0);
            core_renderer.renderer.render(
                layout_state,
                frame_buffer.as_mut_slice(),
                szu32,
                w as u32,
                false,
            );
            let frame_buffer = Cow::Borrowed(frame_buffer.as_slice());
            let image = glium::texture::RawImage2d {
                data: frame_buffer,
                width: w,
                height: h,
                format: u8::rgba_format(),
            };
            let texture = glium::texture::Texture2d::with_mipmaps(
                &self.context,
                image,
                glium::texture::MipmapsOption::NoMipmap,
            )
            .expect("create texture");
            let sampler = texture
                .sampled()
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
                .minify_filter(glium::uniforms::MinifySamplerFilter::Nearest);

            let vertex_buffer = VertexBuffer::new(
                &self.context,
                &[
                    Vertex {
                        // top right
                        a_pos: [w as f32, 0.],
                        a_tc: [1., 0.],
                    },
                    Vertex {
                        // bottom right
                        a_pos: [w as f32, h as f32],
                        a_tc: [1., 1.],
                    },
                    Vertex {
                        // bottom left
                        a_pos: [0., h as f32],
                        a_tc: [0., 1.],
                    },
                    Vertex {
                        // top left
                        a_pos: [0., 0.],
                        a_tc: [0., 0.],
                    },
                ],
            )
            .expect("Create vertex buffer");

            let uniforms = uniform! {
                u_screen_size: [w as f32, h as f32],
                u_sampler: sampler,
            };

            frame.clear_color(0., 0., 0., 1.);
            frame
                .draw(
                    &vertex_buffer,
                    &self.index_buffer,
                    &self.program,
                    &uniforms,
                    &Default::default(),
                )
                .expect("frame draw");
            frame.finish().expect("frame finish drawing");
        }
    }
}

pub struct GliumGLArea {
    pub renderer: RefCell<Option<Renderer>>,
    // TODO: is there a way to give just mod access?
    pub core_renderer: RefCell<Option<Rc<RefCell<LiveSplitCoreRenderer>>>>,
    pub frame_rate: RefCell<f32>,
    pub previous_draw_instant: RefCell<Instant>,
}

impl Default for GliumGLArea {
    fn default() -> Self {
        GliumGLArea {
            renderer: Default::default(),
            core_renderer: Default::default(),
            frame_rate: RefCell::new(crate::appconfig::DEFAULT_FRAME_RATE),
            previous_draw_instant: RefCell::new(Instant::now()),
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for GliumGLArea {
    const NAME: &'static str = "GliumGLArea";
    type Type = super::GliumGLArea;
    type ParentType = gtk::GLArea;
}

impl ObjectImpl for GliumGLArea {}

impl WidgetImpl for GliumGLArea {
    fn realize(&self) {
        self.parent_realize();

        let widget = self.obj();
        if widget.error().is_some() {
            return;
        }

        // SAFETY: we know the GdkGLContext exists as we checked for errors above, and we haven't
        // done any operations on it which could lead to glium's state mismatch. (In theory, GTK
        // doesn't do any state-breaking operations on the context either.)
        //
        // We will also ensure glium's context does not outlive the GdkGLContext by destroying it in
        // `unrealize()`.
        let context =
            unsafe { glium::backend::Context::new(widget.clone(), true, Default::default()) }
                .expect("Create gl context");
        *self.renderer.borrow_mut() = Some(Renderer::new(context));
        widget.use_refresh_timer();
    }

    fn unrealize(&self) {
        *self.renderer.borrow_mut() = None;
        *self.core_renderer.borrow_mut() = None;

        self.parent_unrealize();
    }
}

impl GLAreaImpl for GliumGLArea {
    fn render(&self, _context: &gtk::gdk::GLContext) -> bool {
        let core_renderer = &*self.core_renderer.borrow_mut();
        if let Some(core_renderer) = core_renderer {
            self.renderer
                .borrow()
                .as_ref()
                .expect("borrow renderer")
                .draw(&mut (*core_renderer).borrow_mut());
        }

        true
    }
}
