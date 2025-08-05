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

// TODO: add a destroy method that is called from OpenGLResources::destroy()
#[derive(Debug)]
struct PixelBuffer {
    native_buffer: eframe::glow::NativeBuffer,
    size: [usize; 2],
    mapped: bool,
}

// TODO: add a destroy method that is called from GlowCanvas::destroy()
#[derive(Debug)]
struct OpenGLResources {
    program: eframe::glow::Program,
    u_screen_size: eframe::glow::UniformLocation,
    u_sampler: eframe::glow::UniformLocation,
    vbo: eframe::glow::Buffer,
    vao: eframe::glow::VertexArray,
    element_array_buffer: eframe::glow::Buffer,
    texture: eframe::glow::Texture,
    pixel_buffer: Option<[PixelBuffer; 2]>,
    buffer_idx: usize,
    vertices: [Vertex; 4],
    indices: [u32; 6],
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
        gl: &std::sync::Arc<eframe::glow::Context>,
        update: F,
    ) where
        F: FnOnce(&mut [u8], [u32; 2], u32),
    {
        let sz = viewport.size();
        let sz = [sz.x as usize, sz.y as usize];
        unsafe {
            use eframe::glow::HasContext;
            let mut resources_lock = self.opengl_resources.write().unwrap();
            if let Some(resources) = resources_lock.as_mut() {
                let buffer = resources
                    // opengl resources need to exist by now or we're in big trouble
                    .pixel_buffer
                    .get_or_insert_with(|| {
                        let buf1 = gl.create_buffer().unwrap();
                        debug_assert_eq!(gl.get_error(), 0);
                        let buf2 = gl.create_buffer().unwrap();
                        debug_assert_eq!(gl.get_error(), 0);
                        let buffers = [
                            PixelBuffer {
                                native_buffer: buf1,
                                size: sz,
                                mapped: false,
                            },
                            PixelBuffer {
                                native_buffer: buf2,
                                size: sz,
                                mapped: false,
                            },
                        ];
                        for b in &buffers {
                            gl.bind_buffer(glow::PIXEL_UNPACK_BUFFER, Some(b.native_buffer));
                            debug_assert_eq!(gl.get_error(), 0);
                            gl.buffer_data_size(
                                glow::PIXEL_UNPACK_BUFFER,
                                (sz[0] * sz[1] * 4) as _,
                                glow::STREAM_DRAW,
                            );
                            debug_assert_eq!(gl.get_error(), 0);
                            gl.bind_buffer(glow::PIXEL_UNPACK_BUFFER, None);
                            debug_assert_eq!(gl.get_error(), 0);
                        }
                        buffers
                    });
                debug_assert_eq!(gl.get_error(), 0);
                resources.buffer_idx = (resources.buffer_idx + 1) % 2;
                let next_idx = (resources.buffer_idx + 1) % 2;
                gl.bind_buffer(
                    glow::PIXEL_UNPACK_BUFFER,
                    Some(buffer[resources.buffer_idx].native_buffer),
                );
                debug_assert_eq!(gl.get_error(), 0);
                gl.bind_texture(glow::TEXTURE_2D, Some(resources.texture));
                debug_assert_eq!(gl.get_error(), 0);
                // This logic has become a mess because we need two separate code paths
                // for each buffer (2 buffers). There's an optimization we can apply here,
                // if the texture has been used at the current size previously then we can
                // avoid allocating it and just copy the data back into it 'tex_sub_image_2d`
                // instead of the potentially more expensive `tex_image_2d`.
                // To deal with this mess, we have a flag to manage to tell when a buffer
                // needs to be resized and are over conservative when resetting it. Really
                // the common case we care about is when the window is not being resized
                // and as long as we're handling that case well a bit of extra resizing is fine.
                let buffer_size = buffer[resources.buffer_idx].size;
                let need_to_resize = sz[0] != buffer_size[0] || sz[1] != buffer_size[1];
                if need_to_resize {
                    buffer[0].mapped = false;
                    buffer[1].mapped = false;
                }
                if !buffer[resources.buffer_idx].mapped {
                    gl.tex_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        glow::RGBA8 as _,
                        buffer_size[0] as _,
                        buffer_size[1] as _,
                        0,
                        glow::RGBA,
                        glow::UNSIGNED_BYTE,
                        glow::PixelUnpackData::Slice(None),
                    );
                    debug_assert_eq!(gl.get_error(), 0);
                    buffer[resources.buffer_idx].mapped = true;
                } else {
                    gl.tex_sub_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        0,
                        0,
                        buffer_size[0] as _,
                        buffer_size[1] as _,
                        glow::RGBA,
                        glow::UNSIGNED_BYTE,
                        eframe::glow::PixelUnpackData::BufferOffset(0),
                    );
                    debug_assert_eq!(gl.get_error(), 0);
                }
                // Now we want to make sure that we're setting up the next frame's texture buffer
                // (next here really means the current frame, because the double buffering equates
                // to 1 frame of lag), has the correct size everywhere. It should always be the
                // current viewport size. Anything else is an inconsistency.
                buffer[next_idx].size = sz;
                gl.bind_buffer(
                    glow::PIXEL_UNPACK_BUFFER,
                    Some(buffer[next_idx].native_buffer),
                );
                debug_assert_eq!(gl.get_error(), 0);
                let buf_size = buffer[next_idx].size[0] * buffer[next_idx].size[1] * 4;
                gl.buffer_data_size(glow::PIXEL_UNPACK_BUFFER, buf_size as _, glow::STREAM_DRAW);
                debug_assert_eq!(gl.get_error(), 0);
                let perms = glow::MAP_WRITE_BIT;
                let ptr = gl.map_buffer_range(glow::PIXEL_UNPACK_BUFFER, 0, buf_size as _, perms);
                assert_ne!(ptr, std::ptr::null_mut());
                let slice = std::slice::from_raw_parts_mut(ptr, buf_size);
                debug_assert_eq!(gl.get_error(), 0);
                update(
                    slice,
                    [buffer[next_idx].size[0] as _, buffer[next_idx].size[1] as _],
                    buffer[next_idx].size[0] as _,
                );
                debug_assert_eq!(gl.get_error(), 0);
                gl.unmap_buffer(glow::PIXEL_UNPACK_BUFFER);
                debug_assert_eq!(gl.get_error(), 0);
                gl.bind_buffer(glow::PIXEL_UNPACK_BUFFER, None);
                debug_assert_eq!(gl.get_error(), 0);
                gl.bind_texture(glow::TEXTURE_2D, None);
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
    pub fn destroy(&self, gl: &eframe::glow::Context) {
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
                if let Some(buffers) = opengl.pixel_buffer.as_ref() {
                    for b in buffers {
                        gl.delete_buffer(b.native_buffer);
                        debug_assert!(gl.get_error() == 0, "1");
                    }
                };
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

#[allow(dead_code)]
fn gl_debug(source: u32, typ: u32, id: u32, severity: u32, message: &str) {
    let source_name = match source {
        glow::DEBUG_SOURCE_API => "API",
        glow::DEBUG_SOURCE_OTHER => "Other",
        glow::DEBUG_SOURCE_APPLICATION => "Application",
        glow::DEBUG_SOURCE_THIRD_PARTY => "Third Party",
        glow::DEBUG_SOURCE_WINDOW_SYSTEM => "Window System",
        glow::DEBUG_SOURCE_SHADER_COMPILER => "Shader Compiler",
        _ => "Unknown",
    };
    let type_name = match typ {
        glow::DEBUG_TYPE_ERROR => "Error",
        glow::DEBUG_TYPE_OTHER => "Other",
        glow::DEBUG_TYPE_MARKER => "Marker",
        glow::DEBUG_TYPE_PORTABILITY => "Portability",
        glow::DEBUG_TYPE_POP_GROUP => "Pop group",
        glow::DEBUG_TYPE_PUSH_GROUP => "Push group",
        glow::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "undefined behavior",
        glow::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "deprecated behavior",
        glow::DEBUG_TYPE_PERFORMANCE => "performance",
        _ => "Unknown",
    };
    let severity_name = match severity {
        glow::DEBUG_SEVERITY_LOW => "Low",
        glow::DEBUG_SEVERITY_HIGH => "High",
        glow::DEBUG_SEVERITY_MEDIUM => "Medium",
        glow::DEBUG_SEVERITY_NOTIFICATION => "Notification",
        _ => "Unknown",
    };
    if typ == glow::DEBUG_TYPE_ERROR
        || typ == glow::DEBUG_TYPE_UNDEFINED_BEHAVIOR
        || typ == glow::DEBUG_TYPE_DEPRECATED_BEHAVIOR
        || typ == glow::DEBUG_TYPE_PORTABILITY
    {
        println!(
            "source: {source_name}, type: {type_name}, id: {id}, severity: {severity_name}: {message}"
        );
        panic!();
    }
}

unsafe fn init_gl_resources(gl: &eframe::glow::Context) -> OpenGLResources {
    unsafe {
        use eframe::glow::HasContext;
        debug_assert_eq!(gl.get_error(), 0);
        //gl.enable(glow::DEBUG_OUTPUT);
        debug_assert_eq!(gl.get_error(), 0);
        //gl.debug_message_callback(gl_debug);
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
        use egui::Pos2;
        OpenGLResources {
            element_array_buffer,
            program,
            texture,
            u_sampler,
            u_screen_size,
            vao,
            vbo,
            pixel_buffer: None,
            buffer_idx: 0,
            vertices: [
                Vertex {
                    // top right
                    pos: Pos2::new(1.0, 0.0),
                    uv: Pos2::new(1.0, 0.0),
                },
                Vertex {
                    // bottom right
                    pos: Pos2::new(1.0, 1.0),
                    uv: Pos2::new(1.0, 1.0),
                },
                Vertex {
                    // bottom left
                    pos: Pos2::new(0.0, 1.0),
                    uv: Pos2::new(0.0, 1.0),
                },
                Vertex {
                    // top left
                    pos: Pos2::new(0.0, 0.0),
                    uv: Pos2::new(0.0, 0.0),
                },
            ],
            indices: [
                0, 1, 3, // first triangle
                1, 2, 3, // second triangle
            ],
        }
    }
}

unsafe fn paint_lowlevel(
    gl_ctx: &std::sync::Arc<std::sync::RwLock<Option<OpenGLResources>>>,
    viewport: egui::Rect,
    gl: &eframe::glow::Context,
    texture_resized: bool,
) {
    use eframe::glow::HasContext;
    let w = viewport.max.x - viewport.min.x;
    let h = viewport.max.y - viewport.min.y;
    let mut ctx = gl_ctx.write().unwrap();
    let ctx = ctx.as_mut().unwrap();

    unsafe {
        //let timer = std::time::Instant::now();
        //let gl = frame.gl().expect("Rendering context");
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
        if texture_resized {
            ctx.vertices[0].pos.x = viewport.max.x;
            ctx.vertices[0].pos.y = viewport.min.y;
            ctx.vertices[1].pos.x = viewport.max.x;
            ctx.vertices[1].pos.y = viewport.max.y;
            ctx.vertices[2].pos.x = viewport.min.x;
            ctx.vertices[2].pos.y = viewport.max.y;
            ctx.vertices[3].pos.x = viewport.min.x;
            ctx.vertices[3].pos.y = viewport.min.y;
        }

        debug_assert_eq!(gl.get_error(), 0);
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(ctx.vbo));
        debug_assert_eq!(gl.get_error(), 0);
        gl.buffer_data_u8_slice(
            glow::ARRAY_BUFFER,
            bytemuck::cast_slice(ctx.vertices.as_slice()),
            glow::STREAM_DRAW,
        );
        debug_assert_eq!(gl.get_error(), 0);
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ctx.element_array_buffer));
        debug_assert_eq!(gl.get_error(), 0);
        gl.buffer_data_u8_slice(
            glow::ELEMENT_ARRAY_BUFFER,
            bytemuck::cast_slice(ctx.indices.as_slice()),
            glow::STREAM_DRAW,
        );
        debug_assert_eq!(gl.get_error(), 0);
        gl.draw_elements(
            glow::TRIANGLES,
            ctx.indices.len() as i32,
            glow::UNSIGNED_INT,
            0,
        );
        debug_assert_eq!(gl.get_error(), 0);
        gl.bind_buffer(glow::PIXEL_UNPACK_BUFFER, None);
        debug_assert_eq!(gl.get_error(), 0);
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
