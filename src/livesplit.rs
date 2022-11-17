use crate::appconfig::{AppConfig, YesOrNo};
use crate::autosplitters::supermetroid::Settings;
use crate::utils::*;
use eframe::egui;
use livesplit_core::{Layout, SharedTimer, Timer};
use livesplit_hotkey::Hook;
use memoffset::offset_of;
use parking_lot::RwLock;
use std::error::Error;
use std::sync::Arc;
// TODO: use these on windows too
use thread_priority::{set_current_thread_priority, ThreadPriority};

pub struct LiveSplitCoreRenderer {
    pub frame_buffer: Vec<u8>,
    pub layout: Layout,
    pub renderer: livesplit_core::rendering::software::BorrowedRenderer,
    pub layout_state: Option<livesplit_core::layout::LayoutState>,
    pub timer: SharedTimer,
    pub show_settings_editor: bool,
    pub settings: Arc<RwLock<Settings>>,
    pub can_exit: bool,
    pub is_exiting: bool,
    pub thread_chan: std::sync::mpsc::SyncSender<ThreadEvent>,
    pub project_dirs: directories::ProjectDirs,
    pub app_config: Arc<RwLock<AppConfig>>,
    pub app_config_processed: bool,
    pub opengl_resources: Option<OpenGLResources>,
    pub global_hotkey_hook: Option<Hook>,
}

impl LiveSplitCoreRenderer {
    pub fn confirm_save(&mut self, gl: &std::rc::Rc<glow::Context>) {
        use native_dialog::{MessageDialog, MessageType};
        let empty_path = "".to_owned();
        let document_dir = match directories::UserDirs::new() {
            None => empty_path,
            Some(d) => match d.document_dir() {
                None => empty_path,
                Some(d) => d.to_str().unwrap_or("").to_owned(),
            },
        };
        // TODO: fix this unwrap
        if self.timer.read().unwrap().run().has_been_modified() {
            let save_requested = MessageDialog::new()
                .set_type(MessageType::Error)
                .set_title("Error")
                .set_text("Splits have been modified. Save splits?")
                .show_confirm()
                .unwrap();
            if save_requested {
                self.save_splits_dialog(&document_dir);
            }
        }
        if self.settings.read().has_been_modified() {
            let save_requested = MessageDialog::new()
                .set_type(MessageType::Error)
                .set_title("Error")
                .set_text("Autosplit config may have been modified. Save autosplitter config?")
                .show_confirm()
                .unwrap();
            if save_requested {
                self.save_autosplitter_dialog(&document_dir);
            }
        }
        self.can_exit = true;
        unsafe {
            if let Some(opengl) = &self.opengl_resources {
                use glow::HasContext;
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
                self.opengl_resources = None;
            }
        }
    }

    pub fn save_app_config(&self) {
        messagebox_on_error(|| {
            use std::io::Write;
            let mut config_path = self.project_dirs.preference_dir().to_path_buf();
            config_path.push("settings.toml");
            println!("Saving to {:#?}", config_path);
            let f = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(config_path)?;
            let mut writer = std::io::BufWriter::new(f);
            let toml = toml::to_string_pretty(&(*self.app_config.read()))?;
            writer.write_all(toml.as_bytes())?;
            writer.flush()?;
            Ok(())
        });
    }

    pub fn load_app_config(&mut self) {
        messagebox_on_error(|| {
            use std::io::Read;
            let mut config_path = self.project_dirs.preference_dir().to_path_buf();
            config_path.push("settings.toml");
            println!("Loading from {:#?}", config_path);
            let saved_config: AppConfig = std::fs::File::open(config_path)
                .and_then(|mut f| {
                    let mut buffer = String::new();
                    f.read_to_string(&mut buffer)?;
                    Ok(toml::from_str(&buffer)?)
                })
                .unwrap_or_default();
            // Let the CLI options take precedent if any provided
            // TODO: this logic is bad, I really need to know if the CLI
            // stuff was present and whether the stuff was present in the config
            // but instead I just see two different states that need to be merged.
            let cli_config = self.app_config.clone();
            self.app_config = Arc::new(parking_lot::lock_api::RwLock::new(saved_config));
            if cli_config.read().recent_layout.is_some() {
                self.app_config.write().recent_layout = cli_config.read().recent_layout.clone();
            }
            if cli_config.read().recent_splits.is_some() {
                self.app_config.write().recent_splits = cli_config.read().recent_splits.clone();
            }
            if cli_config.read().recent_autosplitter.is_some() {
                self.app_config.write().recent_autosplitter =
                    cli_config.read().recent_autosplitter.clone();
            }
            if cli_config.read().use_autosplitter.is_some() {
                self.app_config.write().use_autosplitter = cli_config.read().use_autosplitter;
            }
            if cli_config.read().frame_rate.is_some() {
                self.app_config.write().frame_rate = cli_config.read().frame_rate;
            }
            if cli_config.read().polling_rate.is_some() {
                self.app_config.write().polling_rate = cli_config.read().polling_rate;
            }
            if cli_config.read().reset_on_reset.is_some() {
                self.app_config.write().reset_on_reset = cli_config.read().reset_on_reset;
            }
            if cli_config.read().global_hotkeys.is_some() {
                self.app_config.write().global_hotkeys = cli_config.read().global_hotkeys;
            }
            Ok(())
        });
    }

    pub fn process_app_config(&mut self, frame: &mut eframe::Frame) {
        messagebox_on_error(|| {
            // Now that we've converged on a config, try loading what we can
            let app_config = self.app_config.clone();
            if let Some(layout) = &app_config.read().recent_layout {
                let f = std::fs::File::open(layout)?;
                self.load_layout(f, frame)?;
            }
            let app_config = self.app_config.clone();
            if let Some(splits) = &app_config.read().recent_splits {
                let f = std::fs::File::open(splits)?;
                let path = std::path::Path::new(splits)
                    .parent()
                    .ok_or("failed to find parent directory")?;
                self.load_splits(f, path.to_path_buf())?;
            }
            let app_config = self.app_config.clone();
            if let Some(autosplitter) = &app_config.read().recent_autosplitter {
                let f = std::fs::File::open(autosplitter)?;
                self.load_autosplitter(f)?;
            }
            Ok(())
        });
    }

    pub fn load_layout(
        &mut self,
        f: std::fs::File,
        frame: &mut eframe::Frame,
    ) -> Result<(), Box<dyn Error>> {
        use std::io::Read;
        let mut reader = std::io::BufReader::new(f);
        let mut layout_file = String::new();
        reader.read_to_string(&mut layout_file)?;

        self.layout = livesplit_core::layout::parser::parse(&layout_file)?;
        let doc = roxmltree::Document::parse(&layout_file)?;
        doc.root().children().for_each(|d| {
            if d.tag_name().name() == "Layout" {
                use std::str::FromStr;
                let mut mode = None;
                let mut x = None;
                let mut y = None;
                let mut width = None;
                let mut height = None;
                d.children().for_each(|d| {
                    if d.tag_name().name() == "Mode" {
                        mode = d.text();
                    }
                    if d.tag_name().name() == "X" {
                        x = d.text().and_then(|d| f32::from_str(d).ok());
                    }
                    if d.tag_name().name() == "Y" {
                        y = d.text().and_then(|d| f32::from_str(d).ok());
                    }
                    if mode.is_some() && d.tag_name().name() == format!("{}Width", mode.unwrap()) {
                        width = d.text().and_then(|d| f32::from_str(d).ok());
                    }
                    if mode.is_some() && d.tag_name().name() == format!("{}Height", mode.unwrap()) {
                        height = d.text().and_then(|d| f32::from_str(d).ok());
                    }
                    if let (Some(x), Some(y), Some(width), Some(height)) = (x, y, width, height) {
                        frame.set_window_size(egui::Vec2::new(width, height));
                        frame.set_window_pos(egui::Pos2::new(x, y));
                    }
                });
            }
        });
        Ok(())
    }

    pub fn load_splits(
        &mut self,
        f: std::fs::File,
        path: std::path::PathBuf,
    ) -> Result<(), Box<dyn Error>> {
        use livesplit_core::run::parser::composite;
        use std::io::Read;
        let file_contents: Result<Vec<_>, _> = f.bytes().collect();
        // TODO: fix this unwrap
        *self.timer.write().unwrap() = Timer::new(
            composite::parse(
                &file_contents?,
                path.parent().map(|p| p.to_path_buf()),
                true,
            )?
            .run,
        )?;
        Ok(())
    }

    pub fn load_autosplitter(&mut self, f: std::fs::File) -> Result<(), Box<dyn Error>> {
        *self.settings.write() = serde_json::from_reader(std::io::BufReader::new(f))?;
        Ok(())
    }

    pub fn save_splits_dialog(&mut self, default_dir: &str) {
        // TODO: fix this unwrap
        let mut fname = self.timer.read().unwrap().run().extended_file_name(false);
        let app_config = self.app_config.clone();
        let app_config_read = app_config.read();
        let splits = app_config_read.recent_splits.as_ref().unwrap_or_else(|| {
            if fname.is_empty() {
                fname += "annelid.lss";
            } else {
                fname += ".lss";
            }
            &fname
        });
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .read()
            .recent_splits
            .as_ref()
            .map_or(default_path_buf.clone(), |p| {
                let path = std::path::Path::new(&p);
                path.parent().map_or(default_path_buf, |p| p.to_path_buf())
            })
            .into_os_string()
            .into_string()
            .expect("utf8");
        self.save_dialog(
            &dir,
            &splits.clone(),
            ("LiveSplit Splits", "lss"),
            |me, f| {
                use livesplit_core::run::saver::livesplit::IoWrite;
                let writer = IoWrite(&f);
                // TODO: fix this unwrap
                livesplit_core::run::saver::livesplit::save_timer(
                    &me.timer.read().unwrap(),
                    writer,
                )?;
                Ok(())
            },
        );
    }

    pub fn save_autosplitter_dialog(&mut self, default_dir: &str) {
        // TODO: fix this unwrap
        let mut fname = self.timer.read().unwrap().run().extended_file_name(false);
        let app_config = self.app_config.clone();
        let app_config_read = app_config.read();
        let autosplitter = app_config_read
            .recent_autosplitter
            .as_ref()
            .unwrap_or_else(|| {
                if fname.is_empty() {
                    fname += "annelid.asc";
                } else {
                    fname += ".asc";
                }
                &fname
            });
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .read()
            .recent_autosplitter
            .as_ref()
            .map_or(default_path_buf.clone(), |p| {
                let path = std::path::Path::new(&p);
                path.parent().map_or(default_path_buf, |p| p.to_path_buf())
            })
            .into_os_string()
            .into_string()
            .expect("utf8");
        self.save_dialog(
            &dir,
            &autosplitter.clone(),
            ("Autosplitter Configuration", "asc"),
            |me, f| {
                serde_json::to_writer(&f, &*me.settings.read())?;
                Ok(())
            },
        );
    }

    pub fn save_dialog(
        &mut self,
        default_dir: &str,
        default_fname: &str,
        file_type: (&str, &str),
        save_action: impl FnOnce(&mut Self, std::fs::File) -> Result<(), Box<dyn Error>>,
    ) {
        use native_dialog::FileDialog;
        messagebox_on_error(|| {
            let path = FileDialog::new()
                .set_location(default_dir)
                .set_filename(default_fname)
                .add_filter(file_type.0, &[file_type.1])
                .add_filter("Any file", &["*"])
                .show_save_single_file()?;
            let path = match path {
                Some(path) => path,
                None => return Ok(()),
            };
            let f = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)?;
            save_action(self, f)?;
            Ok(())
        });
    }

    pub fn open_layout_dialog(&mut self, default_dir: &str, frame: &mut eframe::Frame) {
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .read()
            .recent_layout
            .as_ref()
            .map_or(default_path_buf.clone(), |p| {
                let path = std::path::Path::new(&p);
                path.parent().map_or(default_path_buf, |p| p.to_path_buf())
            })
            .into_os_string()
            .into_string()
            .expect("utf8");
        self.open_dialog(&dir, ("LiveSplit Layout", "lsl"), |me, f, path| {
            me.load_layout(f, frame)?;
            me.app_config.write().recent_layout =
                Some(path.into_os_string().into_string().expect("utf8"));
            Ok(())
        });
    }

    pub fn open_splits_dialog(&mut self, default_dir: &str) {
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .read()
            .recent_splits
            .as_ref()
            .map_or(default_path_buf.clone(), |p| {
                let path = std::path::Path::new(&p);
                path.parent().map_or(default_path_buf, |p| p.to_path_buf())
            })
            .into_os_string()
            .into_string()
            .expect("utf8");
        self.open_dialog(&dir, ("LiveSplit Splits", "lss"), |me, f, path| {
            me.load_splits(f, path.clone())?;
            me.app_config.write().recent_splits =
                Some(path.into_os_string().into_string().expect("utf8"));
            Ok(())
        });
    }

    pub fn open_autosplitter_dialog(&mut self, default_dir: &str) {
        let default_path_buf = std::path::Path::new(default_dir).to_path_buf();
        let dir = self
            .app_config
            .read()
            .recent_autosplitter
            .as_ref()
            .map_or(default_path_buf.clone(), |p| {
                let path = std::path::Path::new(&p);
                path.parent().map_or(default_path_buf, |p| p.to_path_buf())
            })
            .into_os_string()
            .into_string()
            .expect("utf8");
        self.open_dialog(
            &dir,
            ("Autosplitter Configuration", "asc"),
            |me, f, path| {
                me.load_autosplitter(f)?;
                me.app_config.write().recent_autosplitter =
                    Some(path.into_os_string().into_string().expect("utf8"));
                Ok(())
            },
        );
    }

    pub fn open_dialog(
        &mut self,
        default_dir: &str,
        file_type: (&str, &str),
        open_action: impl FnOnce(
            &mut Self,
            std::fs::File,
            std::path::PathBuf,
        ) -> Result<(), Box<dyn Error>>,
    ) {
        use native_dialog::FileDialog;
        messagebox_on_error(|| {
            let path = FileDialog::new()
                .set_location(&default_dir)
                .add_filter(file_type.0, &[file_type.1])
                .add_filter("Any file", &["*"])
                .show_open_single_file()?;
            let path = match path {
                Some(path) => path,
                None => return Ok(()),
            };
            let f = std::fs::File::open(path.clone())?;
            open_action(self, f, path)?;
            Ok(())
        });
    }

    pub fn enable_global_hotkeys(&mut self) -> Result<(), Box<dyn Error>> {
        // It would be more elegant to use get_or_insert_with, however
        // the `with` branch cannot have a `Result` type if we do that.
        let hook: &Hook = match self.global_hotkey_hook.as_ref() {
            None => {
                self.global_hotkey_hook = Some(Hook::new()?);
                self.global_hotkey_hook.as_ref().unwrap()
            }
            Some(h) => h,
        };
        print!("Registering global hotkeys...");
        // TODO: this is kind of gross because of the logical duplication
        // between egui input handling and global hotkey handling
        // Work is needed to keep them in sync :(
        let timer = self.timer.clone();
        if let Some(hot_key) = self.app_config.read().hot_key_start {
            hook.register(hot_key.to_livesplit_hotkey(), move || {
                // TODO: fix this unwrap
                timer.write().unwrap().split_or_start();
            })?;
        }
        let timer = self.timer.clone();
        // TODO: this is not ideal because if the app_config or thread_chan
        // change after this function is called, these will point to the old
        // values. Probably need to wrap config and thread_chan in Arc
        let config = self.app_config.clone();
        let thread_chan = self.thread_chan.clone();
        if let Some(hot_key) = self.app_config.read().hot_key_reset {
            hook.register(hot_key.to_livesplit_hotkey(), move || {
                // TODO: fix this unwrap
                timer.write().unwrap().reset(true);
                if config.read().use_autosplitter == Some(YesOrNo::Yes) {
                    thread_chan.try_send(ThreadEvent::TimerReset).unwrap_or(());
                }
            })?;
        }
        let timer = self.timer.clone();
        if let Some(hot_key) = self.app_config.read().hot_key_undo {
            hook.register(hot_key.to_livesplit_hotkey(), move || {
                // TODO: fix this unwrap
                timer.write().unwrap().undo_split();
            })?;
        }
        let timer = self.timer.clone();
        if let Some(hot_key) = self.app_config.read().hot_key_skip {
            hook.register(hot_key.to_livesplit_hotkey(), move || {
                // TODO: fix this unwrap
                timer.write().unwrap().skip_split();
            })?;
        }
        let timer = self.timer.clone();
        if let Some(hot_key) = self.app_config.read().hot_key_pause {
            hook.register(hot_key.to_livesplit_hotkey(), move || {
                // TODO: fix this unwrap
                timer.write().unwrap().toggle_pause();
            })?;
        }
        println!("registered");
        Ok(())
    }
}

// TODO: where should this live?
pub enum ThreadEvent {
    TimerReset,
}

pub struct OpenGLResources {
    program: glow::Program,
    u_screen_size: glow::UniformLocation,
    u_sampler: glow::UniformLocation,
    vbo: glow::Buffer,
    vao: glow::VertexArray,
    element_array_buffer: glow::Buffer,
    texture: glow::Texture,
}

#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
#[repr(C)]
struct Vertex {
    pos: epaint::Pos2,
    uv: epaint::Pos2,
}

impl eframe::App for LiveSplitCoreRenderer {
    fn on_exit_event(&mut self) -> bool {
        self.is_exiting = true;
        self.can_exit
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        use glow::HasContext;
        if !self.app_config_processed {
            self.process_app_config(frame);
            self.app_config_processed = true;
            // Since this block should only run once, we abuse it to also
            // set a thread priority only once. We want rendering to take a
            // relative backseat to anything else the user has going on
            // like an emulator.
            set_current_thread_priority(ThreadPriority::Min).unwrap_or(())
        }
        {
            // TODO: please move this to its own method and refactor it....
            let viewport = ctx.input().screen_rect;
            let sz = viewport.size();
            let w = viewport.max.x - viewport.min.x;
            let h = viewport.max.y - viewport.min.y;
            // a local scope so the timer lock has a smaller scope
            // TODO: fix this unwrap
            let timer = self.timer.read().unwrap();
            let snapshot = timer.snapshot();
            match &mut self.layout_state {
                None => {
                    self.layout_state = Some(self.layout.state(&snapshot));
                }
                Some(layout_state) => {
                    self.layout.update_state(layout_state, &snapshot);
                }
            };

            if let Some(layout_state) = &self.layout_state {
                let szu32 = [sz.x as u32, sz.y as u32];
                let sz = [sz.x as usize, sz.y as usize];
                self.frame_buffer.resize(sz[0] * sz[1] * 4, 0);
                self.renderer.render(
                    layout_state,
                    self.frame_buffer.as_mut_slice(),
                    szu32,
                    sz[0] as u32,
                    false,
                );

                //let timer = std::time::Instant::now();
                let gl = frame.gl();
                unsafe {
                    let ctx = self.opengl_resources.get_or_insert_with(|| {
                        let vert = gl.create_shader(glow::VERTEX_SHADER).expect("create vert");
                        debug_assert!(gl.get_error() == 0, "1");
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
                        debug_assert!(gl.get_error() == 0, "1");
                        gl.shader_source(vert, source);
                        debug_assert!(gl.get_error() == 0, "1");
                        gl.compile_shader(vert);
                        debug_assert!(gl.get_error() == 0, "1");
                        debug_assert!(
                            gl.get_shader_compile_status(vert),
                            "{}",
                            gl.get_shader_info_log(vert)
                        );
                        debug_assert!(gl.get_error() == 0, "1");

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
                        debug_assert!(gl.get_error() == 0, "1");
                        gl.shader_source(frag, source);
                        debug_assert!(gl.get_error() == 0, "1");
                        gl.compile_shader(frag);
                        debug_assert!(gl.get_error() == 0, "1");
                        debug_assert!(
                            gl.get_shader_compile_status(frag),
                            "{}",
                            gl.get_shader_info_log(frag)
                        );
                        debug_assert!(gl.get_error() == 0, "1");
                        let program = gl.create_program().expect("create program");
                        debug_assert!(gl.get_error() == 0, "1");
                        gl.attach_shader(program, vert);
                        debug_assert!(gl.get_error() == 0, "1");
                        gl.attach_shader(program, frag);
                        debug_assert!(gl.get_error() == 0, "1");
                        gl.link_program(program);
                        debug_assert!(gl.get_error() == 0, "1");
                        debug_assert!(gl.get_program_link_status(program), "link failed");
                        debug_assert!(gl.get_error() == 0, "1");
                        gl.detach_shader(program, vert);
                        debug_assert!(gl.get_error() == 0, "1");
                        gl.detach_shader(program, frag);
                        debug_assert!(gl.get_error() == 0, "1");
                        gl.delete_shader(vert);
                        debug_assert!(gl.get_error() == 0, "1");
                        gl.delete_shader(frag);
                        debug_assert!(gl.get_error() == 0, "1");
                        let u_screen_size =
                            gl.get_uniform_location(program, "u_screen_size").unwrap();
                        debug_assert!(gl.get_error() == 0, "1");
                        let u_sampler = gl.get_uniform_location(program, "u_sampler").unwrap();
                        debug_assert!(gl.get_error() == 0, "1");

                        let vbo = gl.create_buffer().expect("vbo creation");
                        debug_assert!(gl.get_error() == 0, "1");

                        let a_pos_loc = gl.get_attrib_location(program, "a_pos").unwrap();
                        debug_assert!(gl.get_error() == 0, "1");
                        let a_tc_loc = gl.get_attrib_location(program, "a_tc").unwrap();
                        debug_assert!(gl.get_error() == 0, "1");

                        let stride = std::mem::size_of::<Vertex>() as i32;
                        let vao = gl.create_vertex_array().unwrap();
                        debug_assert!(gl.get_error() == 0, "1");
                        gl.bind_vertex_array(Some(vao));
                        debug_assert!(gl.get_error() == 0, "1");
                        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                        debug_assert!(gl.get_error() == 0, "1");
                        gl.vertex_attrib_pointer_f32(
                            a_pos_loc,
                            2,
                            glow::FLOAT,
                            false,
                            stride,
                            offset_of!(Vertex, pos) as i32,
                        );
                        debug_assert!(gl.get_error() == 0, "1");
                        gl.vertex_attrib_pointer_f32(
                            a_tc_loc,
                            2,
                            glow::FLOAT,
                            false,
                            stride,
                            offset_of!(Vertex, uv) as i32,
                        );
                        debug_assert!(gl.get_error() == 0, "1");
                        debug_assert!(gl.get_error() == 0, "1");

                        let element_array_buffer =
                            gl.create_buffer().expect("create element_array_buffer");
                        debug_assert!(gl.get_error() == 0, "1");
                        let texture = gl.create_texture().expect("create texture");
                        debug_assert!(gl.get_error() == 0, "1");
                        OpenGLResources {
                            element_array_buffer,
                            program,
                            texture,
                            u_sampler,
                            u_screen_size,
                            vao,
                            vbo,
                        }
                    });
                    gl.use_program(Some(ctx.program));
                    gl.bind_vertex_array(Some(ctx.vao));
                    gl.enable_vertex_attrib_array(0);
                    debug_assert!(gl.get_error() == 0, "1");
                    gl.enable_vertex_attrib_array(1);
                    debug_assert!(gl.get_error() == 0, "1");

                    gl.uniform_2_f32(Some(&ctx.u_screen_size), w, h);
                    debug_assert!(gl.get_error() == 0, "1");
                    gl.uniform_1_i32(Some(&ctx.u_sampler), 0);
                    debug_assert!(gl.get_error() == 0, "1");
                    gl.active_texture(glow::TEXTURE0);
                    debug_assert!(gl.get_error() == 0, "1");
                    gl.bind_vertex_array(Some(ctx.vao));
                    debug_assert!(gl.get_error() == 0, "1");
                    gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ctx.element_array_buffer));
                    debug_assert!(gl.get_error() == 0, "1");

                    debug_assert!(gl.get_error() == 0, "1");
                    gl.bind_texture(glow::TEXTURE_2D, Some(ctx.texture));
                    debug_assert!(gl.get_error() == 0, "1");
                    gl.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_MAG_FILTER,
                        glow::NEAREST as _,
                    );
                    debug_assert!(gl.get_error() == 0, "1");
                    gl.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_MIN_FILTER,
                        glow::NEAREST as _,
                    );
                    debug_assert!(gl.get_error() == 0, "1");

                    gl.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_WRAP_S,
                        glow::CLAMP_TO_EDGE as i32,
                    );
                    debug_assert!(gl.get_error() == 0, "1");
                    gl.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_WRAP_T,
                        glow::CLAMP_TO_EDGE as i32,
                    );
                    debug_assert!(gl.get_error() == 0, "1");

                    gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
                    debug_assert!(gl.get_error() == 0, "1");
                    gl.tex_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        glow::SRGB8_ALPHA8 as _,
                        w as _,
                        h as _,
                        0,
                        glow::RGBA,
                        glow::UNSIGNED_BYTE,
                        Some(self.frame_buffer.as_slice()),
                    );
                    debug_assert!(gl.get_error() == 0, "1");

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
                    let indices: Vec<u32> = vec![
                        // note that we start from 0!
                        0, 1, 3, // first triangle
                        1, 2, 3, // second triangle
                    ];

                    debug_assert!(gl.get_error() == 0, "1");
                    gl.bind_buffer(glow::ARRAY_BUFFER, Some(ctx.vbo));
                    debug_assert!(gl.get_error() == 0, "1");
                    gl.buffer_data_u8_slice(
                        glow::ARRAY_BUFFER,
                        bytemuck::cast_slice(vertices.as_slice()),
                        glow::STREAM_DRAW,
                    );
                    debug_assert!(gl.get_error() == 0, "1");
                    gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ctx.element_array_buffer));
                    debug_assert!(gl.get_error() == 0, "1");
                    gl.buffer_data_u8_slice(
                        glow::ELEMENT_ARRAY_BUFFER,
                        bytemuck::cast_slice(indices.as_slice()),
                        glow::STREAM_DRAW,
                    );
                    debug_assert!(gl.get_error() == 0, "1");
                    gl.bind_texture(glow::TEXTURE_2D, Some(ctx.texture));
                    debug_assert!(gl.get_error() == 0, "1");
                    gl.draw_elements(glow::TRIANGLES, indices.len() as i32, glow::UNSIGNED_INT, 0);
                    debug_assert!(gl.get_error() == 0, "1");
                    gl.disable_vertex_attrib_array(0);
                    debug_assert!(gl.get_error() == 0, "1");
                    gl.disable_vertex_attrib_array(1);
                    debug_assert!(gl.get_error() == 0, "1");
                    gl.bind_vertex_array(None);
                    debug_assert!(gl.get_error() == 0, "1");
                    gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
                    debug_assert!(gl.get_error() == 0, "1");
                    gl.bind_buffer(glow::ARRAY_BUFFER, None);
                    debug_assert!(gl.get_error() == 0, "1");
                }
                //println!("Time to render texture: {}μs", timer.elapsed().as_micros());
            }
        }
        ctx.set_visuals(egui::Visuals::dark()); // Switch to dark mode
        let settings_editor = egui::containers::Window::new("Settings Editor");
        egui::Area::new("livesplit")
            .enabled(!self.show_settings_editor)
            .show(ctx, |ui| {
                ui.set_width(ctx.input().screen_rect.width());
                ui.set_height(ctx.input().screen_rect.height());
            })
            .response
            .context_menu(|ui| {
                let empty_path = "".to_owned();
                let document_dir = match directories::UserDirs::new() {
                    None => empty_path,
                    Some(d) => match d.document_dir() {
                        None => empty_path,
                        Some(d) => d.to_str().unwrap_or("").to_owned(),
                    },
                };
                ui.menu_button("LiveSplit Save/Load", |ui| {
                    if ui.button("Import Layout").clicked() {
                        ui.close_menu();
                        self.open_layout_dialog(&document_dir, frame);
                    }
                    if ui.button("Import Splits").clicked() {
                        ui.close_menu();
                        self.open_splits_dialog(&document_dir);
                    }
                    if ui.button("Save Splits as...").clicked() {
                        ui.close_menu();
                        self.save_splits_dialog(&document_dir);
                    }
                });
                ui.menu_button("Run Control", |ui| {
                    if ui.button("Start").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().start();
                        ui.close_menu()
                    }
                    if ui.button("Split").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().split();
                        ui.close_menu()
                    }
                    ui.separator();
                    if ui.button("Skip Split").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().skip_split();
                        ui.close_menu()
                    }
                    if ui.button("Undo Split").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().undo_split();
                        ui.close_menu()
                    }
                    ui.separator();
                    if ui.button("Pause").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().pause();
                        ui.close_menu()
                    }

                    if ui.button("Resume").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().resume();
                        ui.close_menu()
                    }
                    ui.separator();
                    if ui.button("Reset").clicked() {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().reset(true);
                        if self.app_config.read().use_autosplitter == Some(YesOrNo::Yes) {
                            self.thread_chan.send(ThreadEvent::TimerReset).unwrap_or(());
                        }
                        ui.close_menu()
                    }
                });
                ui.menu_button("Autosplitter", |ui| {
                    if ui.button("Configure").clicked() {
                        self.show_settings_editor = true;
                        ui.close_menu();
                    }
                    if ui.button("Load Configuration").clicked() {
                        ui.close_menu();
                        self.open_autosplitter_dialog(&document_dir);
                    }
                    if ui.button("Save Configuration").clicked() {
                        ui.close_menu();
                        self.save_autosplitter_dialog(&document_dir);
                    }
                });
                ui.separator();
                if ui.button("Quit").clicked() {
                    frame.quit();
                }
            });
        settings_editor
            .open(&mut self.show_settings_editor)
            .resizable(true)
            .collapsible(false)
            .hscroll(true)
            .vscroll(true)
            .show(ctx, |ui| {
                ctx.move_to_top(ui.layer_id());
                let mut settings = self.settings.write();
                let mut roots = settings.roots();
                show_children(&mut settings, ui, ctx, &mut roots);
            });
        ctx.input().events.iter().for_each(|e| {
            if let egui::Event::Scroll(v) = e {
                if v.y > 0.0 {
                    self.layout.scroll_up();
                } else {
                    self.layout.scroll_down();
                }
            }
        });
        {
            let app_config = self.app_config.read();
            if app_config.global_hotkeys != Some(YesOrNo::Yes) {
                let mut input = { ctx.input_mut() };
                if let Some(hot_key) = app_config.hot_key_start {
                    if input.consume_key(hot_key.modifiers, hot_key.key) {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().split_or_start();
                    }
                }
                if let Some(hot_key) = app_config.hot_key_reset {
                    if input.consume_key(hot_key.modifiers, hot_key.key) {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().reset(true);
                        if app_config.use_autosplitter == Some(YesOrNo::Yes) {
                            self.thread_chan
                                .try_send(ThreadEvent::TimerReset)
                                .unwrap_or(());
                        }
                    }
                }
                if let Some(hot_key) = app_config.hot_key_undo {
                    if input.consume_key(hot_key.modifiers, hot_key.key) {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().undo_split();
                    }
                }
                if let Some(hot_key) = app_config.hot_key_skip {
                    if input.consume_key(hot_key.modifiers, hot_key.key) {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().skip_split();
                    }
                }
                if let Some(hot_key) = app_config.hot_key_pause {
                    if input.consume_key(hot_key.modifiers, hot_key.key) {
                        // TODO: fix this unwrap
                        self.timer.write().unwrap().toggle_pause();
                    }
                }
            }
        }

        if self.is_exiting {
            self.confirm_save(frame.gl());
            self.save_app_config();
            frame.quit();
        }
    }
}
