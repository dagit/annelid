use memoffset::offset_of;

/// An opengl canvas for efficiently rendering images. Perhaps this should be called `GlowImage`
/// instead, as it doesn't give you a way to modify anything about what is drawn other than a
/// texture. This was created simply because rendering many images per second with `egui::Image`
/// has unacceptable overhead. This bypasses as much of that overhead as possible. You're expected
/// to update the raw bytes of the image manually. by calling `update_frame_buffer`, which will
/// give you a thread safe handle to the frame buffer that you can mutate.
pub struct GlowCanvas {
    opengl_resources: std::sync::Arc<std::sync::RwLock<Option<OpenGLResources>>>,
    sense: egui::Sense,
    previous_texture_size: std::sync::Arc<std::sync::RwLock<(usize, usize)>>,
}

#[derive(Debug)]
struct OpenGLResources {
    program: glow::Program,
    u_screen_size: glow::UniformLocation,
    u_sampler: glow::UniformLocation,
    vbo: glow::Buffer,
    vao: glow::VertexArray,
    element_array_buffer: glow::Buffer,
    texture: glow::Texture,
    pixel_buffer: Option<glow::NativeBuffer>,
}

#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Debug)]
#[repr(C)]
struct Vertex {
    pos: epaint::Pos2,
    uv: epaint::Pos2,
}

impl Default for GlowCanvas {
    fn default() -> Self {
        GlowCanvas {
            opengl_resources: std::sync::Arc::new(std::sync::RwLock::new(None)),
            sense: egui::Sense::click(),
            previous_texture_size: std::sync::Arc::new(std::sync::RwLock::new((0, 0))),
        }
    }
}

impl GlowCanvas {
    pub fn new() -> Self {
        Default::default()
    }

    /// The frame buffer is expected to have an RGBA value for each pixel.
    /// This means that the `frame_buffer.len()` must be 4x the dimensions
    /// of the viewport. Otherwise, drawing the canvas will panic.
    pub fn update_frame_buffer<F>(
        &self,
        viewport: egui::Rect,
        gl: &std::rc::Rc<glow::Context>,
        update: F,
    ) where
        F: FnOnce(&mut [u8], [u32; 2], u32),
    {
        let sz = viewport.size();
        let szu32 = [sz.x as u32, sz.y as u32];
        let sz = [sz.x as usize, sz.y as usize];
        unsafe {
            use glow::HasContext;
            let mut resources_lock = self.opengl_resources.write().unwrap();
            if let Some(resources) = resources_lock.as_mut() {
                let buffer = resources
                    // opengl resources needs to exist by now or we're in big trouble
                    .pixel_buffer
                    .get_or_insert_with(|| {
                        let buf = gl.create_buffer().unwrap();
                        debug_assert_eq!(gl.get_error(), 0);
                        buf
                    });
                debug_assert_eq!(gl.get_error(), 0);
                gl.bind_buffer(glow::PIXEL_UNPACK_BUFFER, Some(*buffer));
                debug_assert_eq!(gl.get_error(), 0);
                gl.buffer_data_size(
                    glow::PIXEL_UNPACK_BUFFER,
                    (sz[0] * sz[1] * 4) as _,
                    glow::STREAM_DRAW,
                );
                let perms = glow::MAP_WRITE_BIT;
                let ptr = gl.map_buffer_range(
                    glow::PIXEL_UNPACK_BUFFER,
                    0,
                    (sz[0] * sz[1] * 4) as _,
                    perms,
                );
                assert_ne!(ptr, std::ptr::null_mut());
                let slice = std::slice::from_raw_parts_mut(ptr, sz[0] * sz[1] * 4);
                debug_assert_eq!(gl.get_error(), 0);
                update(slice, szu32, sz[0] as _);
                gl.unmap_buffer(glow::PIXEL_UNPACK_BUFFER);
                debug_assert_eq!(gl.get_error(), 0);
                gl.bind_buffer(glow::PIXEL_UNPACK_BUFFER, None);
                debug_assert_eq!(gl.get_error(), 0);
            }
        }
    }

    /// Paints immediately using the GL context. In some versions of egui, this painting will be
    /// cleared right after it is drawn. This method exists because in principle it is the lowest
    /// overhead way to draw but if you get a blank area, try a different method.
    pub fn paint_immediate(&self, gl: &std::rc::Rc<eframe::glow::Context>, rect: egui::Rect) {
        let viewport = rect;
        let gl_ctx = self.opengl_resources.clone();
        paint(&gl_ctx, viewport, gl, &self.previous_texture_size);
    }

    /// This uses a paint callback to draw to a chosen layer. Usually, the layer will be `LayerId::background()`.
    /// This should be about the same overhead as `paint_immediate` while using the paint callback
    /// system because it seems to be better supported across egui versions.
    pub fn paint_layer(&self, ctx: &egui::Context, layer: egui::LayerId, rect: egui::Rect) {
        let painter = ctx.layer_painter(layer);
        let viewport = rect;
        let gl_ctx = self.opengl_resources.clone();
        let prev_size = self.previous_texture_size.clone();
        let callback = egui::PaintCallback {
            rect: viewport,
            callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                paint(&gl_ctx, viewport, painter.gl(), &prev_size);
            })),
        };

        painter.add(callback);
    }

    /// This works similarly to `Image::paint_at()` but it's lightly tested and may have issues
    /// with layout.
    pub fn paint_at(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let (rect, _response) = ui.allocate_exact_size(rect.size(), self.sense);
        if ui.is_rect_visible(rect) {
            let viewport = rect;
            let gl_ctx = self.opengl_resources.clone();
            let prev_size = self.previous_texture_size.clone();
            let callback = egui::PaintCallback {
                rect: viewport,
                callback: std::sync::Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                    paint(&gl_ctx, viewport, painter.gl(), &prev_size);
                })),
            };

            ui.painter().add(callback);
        }
    }

    // TODO: is there a better way to handle this? The tricky bit is that we need a valid
    // opengl context in order to destroy things.
    pub fn destroy(&self, gl: &glow::Context) {
        use eframe::glow::HasContext;
        unsafe {
            if let Some(opengl) = &*self.opengl_resources.read().unwrap() {
                // TODO: is this everything we're supposed to delete?
                gl.delete_texture(opengl.texture);
                debug_assert!(gl.get_error() == 0, "1");
                gl.delete_buffer(opengl.vbo);
                debug_assert!(gl.get_error() == 0, "1");
                gl.delete_buffer(opengl.element_array_buffer);
                debug_assert!(gl.get_error() == 0, "1");
                gl.delete_vertex_array(opengl.vao);
                debug_assert!(gl.get_error() == 0, "1");
                gl.delete_program(opengl.program);
                debug_assert!(gl.get_error() == 0, "1");
                //self.opengl_resources = std::sync::Arc::new(std::sync::RwLock::new(None));
            }
        }
    }
}

/// Safe-ish Wrapper around the low level painting primitives
fn paint(
    gl_ctx: &std::sync::Arc<std::sync::RwLock<Option<OpenGLResources>>>,
    viewport: egui::Rect,
    gl: &eframe::glow::Context,
    previous_size: &std::sync::Arc<std::sync::RwLock<(usize, usize)>>,
) {
    // Wish we could use get_or_insert_with here, but we need to return
    // the Arc<Mutex<_>> instead of just a mut &_
    let gl_ctx = if gl_ctx.read().unwrap().is_some() {
        gl_ctx.clone()
    } else {
        unsafe {
            *gl_ctx.write().unwrap() = Some(init_gl_resources(gl));
            gl_ctx.clone()
        }
    };
    unsafe {
        let mut previous_size = previous_size.write().unwrap();
        let texture_resized = viewport.width() as usize != previous_size.0
            || viewport.height() as usize != previous_size.1;
        paint_lowlevel(&gl_ctx, viewport, gl, texture_resized);
        *previous_size = (viewport.width() as usize, viewport.height() as usize);
    }
}

// Everything after this point is just the low level opengl rendering code

fn gl_debug(source: u32, typ: u32, id: u32, severity: u32, message: &str) {
    println!(
        "source: {}, type: {}, id: {}, severity: {}: {}",
        source, typ, id, severity, message
    );
}

unsafe fn init_gl_resources(gl: &eframe::glow::Context) -> OpenGLResources {
    unsafe {
        use eframe::glow::HasContext;
        debug_assert_eq!(gl.get_error(), 0);
        //gl.enable(glow::DEBUG_OUTPUT);
        debug_assert_eq!(gl.get_error(), 0);
        gl.debug_message_callback(gl_debug);
        debug_assert_eq!(gl.get_error(), 0);
        let vert = gl.create_shader(glow::VERTEX_SHADER).expect("create vert");
        debug_assert_eq!(gl.get_error(), 0);
        let source = "
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
}";
        debug_assert_eq!(gl.get_error(), 0);
        gl.shader_source(vert, source);
        debug_assert_eq!(gl.get_error(), 0);
        gl.compile_shader(vert);
        debug_assert_eq!(gl.get_error(), 0);
        debug_assert!(
            gl.get_shader_compile_status(vert),
            "{}",
            gl.get_shader_info_log(vert)
        );
        debug_assert_eq!(gl.get_error(), 0);

        let frag = gl
            .create_shader(glow::FRAGMENT_SHADER)
            .expect("crate fragment");
        let source = "
#version 330

uniform sampler2D u_sampler;

in      vec2      v_tc;

out     vec4      fragmentColor;

void main() {
    fragmentColor = texture(u_sampler, v_tc);
}
";
        debug_assert_eq!(gl.get_error(), 0);
        gl.shader_source(frag, source);
        debug_assert_eq!(gl.get_error(), 0);
        gl.compile_shader(frag);
        debug_assert_eq!(gl.get_error(), 0);
        debug_assert!(
            gl.get_shader_compile_status(frag),
            "{}",
            gl.get_shader_info_log(frag)
        );
        debug_assert_eq!(gl.get_error(), 0);
        let program = gl.create_program().expect("create program");
        debug_assert_eq!(gl.get_error(), 0);
        gl.attach_shader(program, vert);
        debug_assert_eq!(gl.get_error(), 0);
        gl.attach_shader(program, frag);
        debug_assert_eq!(gl.get_error(), 0);
        gl.link_program(program);
        debug_assert_eq!(gl.get_error(), 0);
        debug_assert!(gl.get_program_link_status(program), "link failed");
        debug_assert_eq!(gl.get_error(), 0);
        gl.detach_shader(program, vert);
        debug_assert_eq!(gl.get_error(), 0);
        gl.detach_shader(program, frag);
        debug_assert_eq!(gl.get_error(), 0);
        gl.delete_shader(vert);
        debug_assert_eq!(gl.get_error(), 0);
        gl.delete_shader(frag);
        debug_assert_eq!(gl.get_error(), 0);
        let u_screen_size = gl.get_uniform_location(program, "u_screen_size").unwrap();
        debug_assert_eq!(gl.get_error(), 0);
        let u_sampler = gl.get_uniform_location(program, "u_sampler").unwrap();
        debug_assert_eq!(gl.get_error(), 0);

        let vbo = gl.create_buffer().expect("vbo creation");
        debug_assert_eq!(gl.get_error(), 0);

        let a_pos_loc = gl.get_attrib_location(program, "a_pos").unwrap();
        debug_assert_eq!(gl.get_error(), 0);
        let a_tc_loc = gl.get_attrib_location(program, "a_tc").unwrap();
        debug_assert_eq!(gl.get_error(), 0);

        let stride = std::mem::size_of::<Vertex>() as i32;
        let vao = gl.create_vertex_array().unwrap();
        debug_assert_eq!(gl.get_error(), 0);
        gl.bind_vertex_array(Some(vao));
        debug_assert_eq!(gl.get_error(), 0);
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        debug_assert_eq!(gl.get_error(), 0);
        gl.vertex_attrib_pointer_f32(
            a_pos_loc,
            2,
            glow::FLOAT,
            false,
            stride,
            offset_of!(Vertex, pos) as i32,
        );
        debug_assert_eq!(gl.get_error(), 0);
        gl.vertex_attrib_pointer_f32(
            a_tc_loc,
            2,
            glow::FLOAT,
            false,
            stride,
            offset_of!(Vertex, uv) as i32,
        );
        debug_assert_eq!(gl.get_error(), 0);

        let element_array_buffer = gl.create_buffer().expect("create element_array_buffer");
        debug_assert_eq!(gl.get_error(), 0);
        let texture = gl.create_texture().expect("create texture");
        debug_assert_eq!(gl.get_error(), 0);
        // We need to do this at least once to set the opengl parameters
        // on the texture so that later we can use tex_sub_image_2d.
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA8 as _,
            0 as _,
            0 as _,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            None,
        );
        debug_assert_eq!(gl.get_error(), 0);
        OpenGLResources {
            element_array_buffer,
            program,
            texture,
            u_sampler,
            u_screen_size,
            vao,
            vbo,
            pixel_buffer: None,
        }
    }
}

unsafe fn paint_lowlevel(
    gl_ctx: &std::sync::Arc<std::sync::RwLock<Option<OpenGLResources>>>,
    viewport: egui::Rect,
    gl: &eframe::glow::Context,
    texture_resized: bool,
) {
    let w = viewport.max.x - viewport.min.x;
    let h = viewport.max.y - viewport.min.y;

    unsafe {
        use eframe::glow::HasContext;
        let ctx = gl_ctx.read().unwrap();
        let ctx = ctx.as_ref().unwrap();

        //let timer = std::time::Instant::now();
        gl.use_program(Some(ctx.program));
        debug_assert_eq!(gl.get_error(), 0);
        gl.bind_vertex_array(Some(ctx.vao));
        debug_assert_eq!(gl.get_error(), 0);
        gl.enable_vertex_attrib_array(0);
        debug_assert_eq!(gl.get_error(), 0);
        gl.enable_vertex_attrib_array(1);
        debug_assert_eq!(gl.get_error(), 0);

        gl.uniform_2_f32(Some(&ctx.u_screen_size), w, h);
        debug_assert_eq!(gl.get_error(), 0);
        gl.uniform_1_i32(Some(&ctx.u_sampler), 0);
        debug_assert_eq!(gl.get_error(), 0);
        gl.active_texture(glow::TEXTURE0);
        debug_assert_eq!(gl.get_error(), 0);
        gl.bind_vertex_array(Some(ctx.vao));
        debug_assert_eq!(gl.get_error(), 0);
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ctx.element_array_buffer));
        debug_assert_eq!(gl.get_error(), 0);

        debug_assert_eq!(gl.get_error(), 0);
        gl.bind_texture(glow::TEXTURE_2D, Some(ctx.texture));
        debug_assert_eq!(gl.get_error(), 0);
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::NEAREST as _,
        );
        debug_assert_eq!(gl.get_error(), 0);
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::NEAREST as _,
        );
        debug_assert_eq!(gl.get_error(), 0);

        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            glow::CLAMP_TO_EDGE as i32,
        );
        debug_assert_eq!(gl.get_error(), 0);
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            glow::CLAMP_TO_EDGE as i32,
        );
        debug_assert_eq!(gl.get_error(), 0);

        gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
        debug_assert_eq!(gl.get_error(), 0);
        //println!("({},{})", w, h);
        let pbo = gl_ctx.read().unwrap().as_ref().unwrap().pixel_buffer;
        if pbo.is_some() {
            gl.bind_buffer(glow::PIXEL_UNPACK_BUFFER, pbo);
            debug_assert_eq!(gl.get_error(), 0);
            if texture_resized {
                gl.tex_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGBA8 as _,
                    w as _,
                    h as _,
                    0,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    None,
                );
                debug_assert_eq!(gl.get_error(), 0);
            } else {
                gl.tex_sub_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    0,
                    0,
                    w as _,
                    h as _,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    glow::PixelUnpackData::BufferOffset(0),
                );
                debug_assert_eq!(gl.get_error(), 0);
            }

            use epaint::Pos2;
            let vertices: Vec<Vertex> = vec![
                Vertex {
                    // top right
                    pos: Pos2::new(viewport.max.x, viewport.min.y),
                    uv: Pos2::new(1.0, 0.0),
                },
                Vertex {
                    // bottom right
                    pos: Pos2::new(viewport.max.x, viewport.max.y),
                    uv: Pos2::new(1.0, 1.0),
                },
                Vertex {
                    // bottom left
                    pos: Pos2::new(viewport.min.x, viewport.max.y),
                    uv: Pos2::new(0.0, 1.0),
                },
                Vertex {
                    // top left
                    pos: Pos2::new(viewport.min.x, viewport.min.y),
                    uv: Pos2::new(0.0, 0.0),
                },
            ];
            //println!("{:#?}", vertices);
            let indices: Vec<u32> = vec![
                // note that we start from 0!
                0, 1, 3, // first triangle
                1, 2, 3, // second triangle
            ];

            debug_assert_eq!(gl.get_error(), 0);
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(ctx.vbo));
            debug_assert_eq!(gl.get_error(), 0);
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(vertices.as_slice()),
                glow::STREAM_DRAW,
            );
            debug_assert_eq!(gl.get_error(), 0);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ctx.element_array_buffer));
            debug_assert_eq!(gl.get_error(), 0);
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                bytemuck::cast_slice(indices.as_slice()),
                glow::STREAM_DRAW,
            );
            debug_assert_eq!(gl.get_error(), 0);
            gl.bind_texture(glow::TEXTURE_2D, Some(ctx.texture));
            debug_assert_eq!(gl.get_error(), 0);
            gl.draw_elements(glow::TRIANGLES, indices.len() as i32, glow::UNSIGNED_INT, 0);
            debug_assert_eq!(gl.get_error(), 0);
        }
        gl.bind_texture(glow::TEXTURE_2D, None);
        debug_assert_eq!(gl.get_error(), 0);
        gl.disable_vertex_attrib_array(0);
        debug_assert_eq!(gl.get_error(), 0);
        gl.disable_vertex_attrib_array(1);
        debug_assert_eq!(gl.get_error(), 0);
        gl.bind_vertex_array(None);
        debug_assert_eq!(gl.get_error(), 0);
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        debug_assert_eq!(gl.get_error(), 0);
        gl.bind_buffer(glow::ARRAY_BUFFER, None);
        debug_assert_eq!(gl.get_error(), 0);
        gl.use_program(None);
        debug_assert_eq!(gl.get_error(), 0);
        //println!("Time to render texture: {}μs", timer.elapsed().as_micros());
    }
}
