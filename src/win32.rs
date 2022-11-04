// This is based almost entirely on the direct2d example in windows-rs
use livesplit_core::{Layout, SharedTimer};
use parking_lot::RwLock;
use std::sync::Arc;
use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct2D::Common::*,
    Win32::Graphics::Direct2D::*, Win32::Graphics::Direct3D::*, Win32::Graphics::Direct3D11::*,
    Win32::Graphics::Dxgi::Common::*, Win32::Graphics::Dxgi::*, Win32::Graphics::Gdi::*,
    Win32::System::Com::*, Win32::System::LibraryLoader::*, Win32::System::Performance::*,
    Win32::UI::WindowsAndMessaging::*,
};

pub fn main(
    layout: Layout,
    timer: SharedTimer,
    renderer: livesplit_core::rendering::software::Renderer,
) -> Result<Window> {
    unsafe {
        CoInitializeEx(std::ptr::null(), COINIT_MULTITHREADED)?;
    }
    Window::new(layout, timer, renderer)
}

pub struct Window {
    handle: Arc<RwLock<HWND>>,
    factory: ID2D1Factory1,
    dxfactory: IDXGIFactory2,
    variable: std::cell::RefCell<u8>,
    target: Option<ID2D1DeviceContext>,
    swapchain: Option<IDXGISwapChain1>,
    dpi: f32,
    visible: bool,
    occlusion: u32,
    layout: Layout,
    timer: SharedTimer,
    renderer: livesplit_core::rendering::software::Renderer,
}

impl Window {
    fn new(
        layout: Layout,
        timer: SharedTimer,
        renderer: livesplit_core::rendering::software::Renderer,
    ) -> Result<Self> {
        let factory = create_factory()?;
        let dxfactory: IDXGIFactory2 = unsafe { CreateDXGIFactory1()? };

        let mut dpi = 0.0;
        let mut dpiy = 0.0;
        unsafe { factory.GetDesktopDpi(&mut dpi, &mut dpiy) };

        let mut frequency = 0;
        unsafe { QueryPerformanceFrequency(&mut frequency).ok()? };

        println!("ok window");
        Ok(Window {
            layout,
            renderer,
            timer,
            handle: Arc::new(RwLock::new(HWND(0))),
            factory,
            dxfactory,
            variable: std::cell::RefCell::new(0),
            target: None,
            swapchain: None,
            dpi,
            visible: false,
            occlusion: 0,
        })
    }

    pub fn handle(&self) -> Arc<RwLock<HWND>> {
        self.handle.clone()
    }

    pub fn render(&mut self) -> Result<()> {
        if self.target.is_none() {
            let device = create_device()?;
            let target = create_render_target(&self.factory, &device)?;
            unsafe { target.SetDpi(self.dpi, self.dpi) };

            let swapchain = create_swapchain(&device, *self.handle.read())?;
            create_swapchain_bitmap(&swapchain, &target)?;

            self.target = Some(target);
            self.swapchain = Some(swapchain);
            self.create_device_size_resources()?;
        }

        let target = self.target.as_ref().unwrap();
        let var = *self.variable.borrow();
        *self.variable.borrow_mut() = (var + 1) % 255;
        unsafe { target.BeginDraw() };
        unsafe {
            target.Clear(&D2D1_COLOR_F {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            })
        };
        let size_f = unsafe { target.GetSize() };
        let size_u = D2D_SIZE_U {
            width: (size_f.width * self.dpi / 96.0) as u32,
            height: (size_f.height * self.dpi / 96.0) as u32,
        };

        let properties = D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_R8G8B8A8_UNORM,
                alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
            },
            dpiX: self.dpi,
            dpiY: self.dpi,
            bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET,
            colorContext: None,
        };
        use winapi::ctypes::c_void;
        //let pixels =
        //    vec![*self.variable.borrow(); size_u.width as usize * size_u.height as usize * 4];
        // a local scope so the timer lock has a smaller scope
        let layout_state = {
	    // TODO: do something better than unwrap
            let timer = self.timer.read().unwrap();
            let snapshot = timer.snapshot();
            self.layout.state(&snapshot)
        };
        self.renderer
            .render(&layout_state, [size_u.width, size_u.height]);
        let raw_frame = self.renderer.image_data();
        let bitmap = unsafe {
            target.CreateBitmap2(
                size_u,
                raw_frame.as_ptr() as *const c_void,
                size_u.width * 4,
                &properties,
            )?
        };
        //let bitmap = unsafe { target.CreateBitmap2(size_u, pixels.as_ptr() as *const c_void, size_u.width * 4, &properties)? };
        self.draw(target, &bitmap)?;

        unsafe {
            target.EndDraw(std::ptr::null_mut(), std::ptr::null_mut())?;
        }

        if let Err(error) = self.present(1, 0) {
            if error.code() == DXGI_STATUS_OCCLUDED {
                self.occlusion = unsafe {
                    self.dxfactory
                        .RegisterOcclusionStatusWindow(*self.handle.read(), WM_USER)?
                };
                self.visible = false;
            } else {
                self.release_device();
            }
        }

        Ok(())
    }

    fn release_device(&mut self) {
        self.target = None;
        self.swapchain = None;
        self.release_device_resources();
    }

    fn release_device_resources(&mut self) {}

    fn present(&self, sync: u32, flags: u32) -> Result<()> {
        unsafe { self.swapchain.as_ref().unwrap().Present(sync, flags).ok() }
    }

    fn draw(&self, target: &ID2D1DeviceContext, bitmap: &ID2D1Bitmap1) -> Result<()> {
        unsafe {
            target.DrawBitmap(
                bitmap,
                std::ptr::null(),
                1.0,
                D2D1_BITMAP_INTERPOLATION_MODE_NEAREST_NEIGHBOR,
                std::ptr::null(),
            );
        }

        Ok(())
    }

    fn create_device_size_resources(&mut self) -> Result<()> {
        Ok(())
    }

    fn resize_swapchain_bitmap(&mut self) -> Result<()> {
        if let Some(target) = &self.target {
            let swapchain = self.swapchain.as_ref().unwrap();
            unsafe { target.SetTarget(None) };

            if unsafe {
                swapchain
                    .ResizeBuffers(0, 0, 0, DXGI_FORMAT_UNKNOWN, 0)
                    .is_ok()
            } {
                create_swapchain_bitmap(swapchain, target)?;
                self.create_device_size_resources()?;
            } else {
                self.release_device();
            }

            self.render()?;
        }

        Ok(())
    }

    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            match message {
                WM_PAINT => {
                    let mut ps = PAINTSTRUCT::default();
                    BeginPaint(*self.handle.read(), &mut ps);
                    self.render().unwrap();
                    EndPaint(*self.handle.read(), &ps);
                    LRESULT(0)
                }
                WM_SIZE => {
                    if wparam.0 != SIZE_MINIMIZED as usize {
                        self.resize_swapchain_bitmap().unwrap();
                    }
                    LRESULT(0)
                }
                WM_DISPLAYCHANGE => {
                    self.render().unwrap();
                    LRESULT(0)
                }
                WM_USER => {
                    if self.present(0, DXGI_PRESENT_TEST).is_ok() {
                        self.dxfactory.UnregisterOcclusionStatus(self.occlusion);
                        self.occlusion = 0;
                        self.visible = true;
                    }
                    LRESULT(0)
                }
                WM_ACTIVATE => {
                    self.visible = true; // TODO: unpack !HIWORD(wparam);
                    LRESULT(0)
                }
                WM_DESTROY => {
                    PostQuitMessage(0);
                    LRESULT(0)
                }
                _ => DefWindowProcA(*self.handle.read(), message, wparam, lparam),
            }
        }
    }

    pub fn run(&mut self) -> Result<()> {
        unsafe {
            let instance = GetModuleHandleA(None)?;
            debug_assert!(instance.0 != 0);
            let window_class = s!("window");

            let wc = WNDCLASSA {
                hCursor: LoadCursorW(None, IDC_HAND)?,
                hInstance: instance,
                lpszClassName: window_class,

                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wndproc),
                ..Default::default()
            };

            let atom = RegisterClassA(&wc);
            debug_assert!(atom != 0);

            let handle = CreateWindowExA(
                WINDOW_EX_STYLE::default(),
                window_class,
                s!("Annelid"),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                None,
                None,
                instance,
                self as *mut _ as _,
            );

            debug_assert!(handle.0 != 0);
            debug_assert!(handle == *self.handle.read());
            let mut message = MSG::default();
	    // TODO: just for testing
	    self.timer.write().unwrap().start();

            loop {
                if self.visible {
                    self.render()?;
                }

                GetMessageA(&mut message, None, 0, 0);

                if message.message == WM_QUIT {
                    return Ok(());
                }

                DispatchMessageA(&message);
            }
        }
    }

    extern "system" fn wndproc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            if message == WM_NCCREATE {
                let cs = lparam.0 as *const CREATESTRUCTA;
                let this = (*cs).lpCreateParams as *mut Self;
                *(*this).handle.write() = window;

                SetWindowLong(window, GWLP_USERDATA, this as _);
            } else {
                let this = GetWindowLong(window, GWLP_USERDATA) as *mut Self;

                if !this.is_null() {
                    return (*this).message_handler(message, wparam, lparam);
                }
            }

            DefWindowProcA(window, message, wparam, lparam)
        }
    }
}

fn create_factory() -> Result<ID2D1Factory1> {
    let mut options = D2D1_FACTORY_OPTIONS::default();

    if cfg!(debug_assertions) {
        options.debugLevel = D2D1_DEBUG_LEVEL_INFORMATION;
    }

    unsafe { D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, &options) }
}

fn create_device_with_type(drive_type: D3D_DRIVER_TYPE) -> Result<ID3D11Device> {
    let mut flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT;

    if cfg!(debug_assertions) {
        flags |= D3D11_CREATE_DEVICE_DEBUG;
    }

    let mut device = None;

    unsafe {
        D3D11CreateDevice(
            None,
            drive_type,
            None,
            flags,
            &[],
            D3D11_SDK_VERSION,
            &mut device,
            std::ptr::null_mut(),
            &mut None,
        )
        .map(|()| device.unwrap())
    }
}

fn create_device() -> Result<ID3D11Device> {
    let mut result = create_device_with_type(D3D_DRIVER_TYPE_HARDWARE);

    if let Err(err) = &result {
        if err.code() == DXGI_ERROR_UNSUPPORTED {
            result = create_device_with_type(D3D_DRIVER_TYPE_WARP);
        }
    }

    result
}

fn create_render_target(
    factory: &ID2D1Factory1,
    device: &ID3D11Device,
) -> Result<ID2D1DeviceContext> {
    unsafe {
        let d2device = factory.CreateDevice(&device.cast::<IDXGIDevice>()?)?;

        let target = d2device.CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;

        target.SetUnitMode(D2D1_UNIT_MODE_DIPS);

        Ok(target)
    }
}

fn get_dxgi_factory(device: &ID3D11Device) -> Result<IDXGIFactory2> {
    let dxdevice = device.cast::<IDXGIDevice>()?;
    unsafe { dxdevice.GetAdapter()?.GetParent() }
}

fn create_swapchain_bitmap(swapchain: &IDXGISwapChain1, target: &ID2D1DeviceContext) -> Result<()> {
    let surface: IDXGISurface = unsafe { swapchain.GetBuffer(0)? };

    let props = D2D1_BITMAP_PROPERTIES1 {
        pixelFormat: D2D1_PIXEL_FORMAT {
            format: DXGI_FORMAT_R8G8B8A8_UNORM,
            alphaMode: D2D1_ALPHA_MODE_IGNORE,
        },
        dpiX: 96.0,
        dpiY: 96.0,
        bitmapOptions: D2D1_BITMAP_OPTIONS_TARGET | D2D1_BITMAP_OPTIONS_CANNOT_DRAW,
        colorContext: None,
    };

    unsafe {
        let bitmap = target.CreateBitmapFromDxgiSurface(&surface, &props)?;
        target.SetTarget(&bitmap);
    };

    Ok(())
}

fn create_swapchain(device: &ID3D11Device, window: HWND) -> Result<IDXGISwapChain1> {
    let factory = get_dxgi_factory(device)?;

    let props = DXGI_SWAP_CHAIN_DESC1 {
        Format: DXGI_FORMAT_R8G8B8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
        BufferCount: 2,
        SwapEffect: DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
        ..Default::default()
    };

    unsafe { factory.CreateSwapChainForHwnd(device, window, &props, std::ptr::null(), None) }
}

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "32")]
unsafe fn SetWindowLong(window: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
    SetWindowLongA(window, index, value as _) as _
}

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "64")]
unsafe fn SetWindowLong(window: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> isize {
    SetWindowLongPtrA(window, index, value)
}

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "32")]
unsafe fn GetWindowLong(window: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    GetWindowLongA(window, index) as _
}

#[allow(non_snake_case)]
#[cfg(target_pointer_width = "64")]
unsafe fn GetWindowLong(window: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    GetWindowLongPtrA(window, index)
}
