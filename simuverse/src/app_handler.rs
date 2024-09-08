use std::{rc::Rc, sync::Mutex, thread};

use crate::SimuverseApp;
use app_surface::AppSurface;
use glam::Vec2;
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use std::time;
#[cfg(target_arch = "wasm32")]
use web_time as time;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, MouseButton, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

pub fn run() -> Result<(), impl std::error::Error> {
    let events_loop = EventLoop::new().unwrap();
    let mut handler = SimuverseAppHandler::default();
    events_loop.run_app(&mut handler)
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{prelude::*, JsCast};

const CANVAS_SIZE_NEED_CHANGE: &str = "canvas_size_need_change";
#[allow(unused)]
const ALL_CUSTOM_EVENTS: [&str; 1] = [CANVAS_SIZE_NEED_CHANGE];

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = Function, js_name = "prototype.call.call")]
    fn call_catch(this: &JsValue) -> Result<(), JsValue>;
    fn canvas_resize_completed();
}

#[cfg(target_arch = "wasm32")]
fn try_call_canvas_resize_completed() {
    let run_closure = Closure::once_into_js(canvas_resize_completed);
    if call_catch(&run_closure).is_err() {
        log::error!("js 端没有定义 canvas_resize_completed 函数");
    }
}

#[allow(unused)]
#[derive(Debug, PartialEq)]
struct CustomJsTriggerEvent {
    ty: &'static str,
    value: String,
}

const WAIT_TIME: time::Duration = time::Duration::from_millis(8);
const POLL_SLEEP_TIME: time::Duration = time::Duration::from_millis(8);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    #[default]
    Wait,
    WaitUntil,
    Poll,
}

#[derive(Default)]
struct SimuverseAppHandler {
    mode: Mode,
    wait_cancelled: bool,
    close_requested: bool,
    last_touch_point: Vec2,
    app: Rc<Mutex<Option<SimuverseApp>>>,
}

impl SimuverseAppHandler {
    fn create_app(&mut self, window: Arc<Window>) {
        // 计算一个默认显示高度
        let height = (if cfg!(target_arch = "wasm32") {
            700.0
        } else {
            750.0
        } * window.scale_factor()) as u32;
        let width = (height as f32 * 1.6) as u32;

        if cfg!(not(target_arch = "wasm32")) {
            let _ = window.request_inner_size(PhysicalSize::new(width, height));
        }

        #[cfg(target_arch = "wasm32")]
        {
            // Winit prevents sizing with CSS, so we have to set
            // the size manually when on web.
            use winit::platform::web::WindowExtWebSys;
            web_sys::window()
                .and_then(|win| win.document())
                .map(|doc| {
                    let canvas = window.canvas().unwrap();
                    let scale_factor = window.scale_factor() as f32;
                    let mut w = width as f32 / scale_factor;
                    let mut h = height as f32 / scale_factor;
                    if let Some(container) = doc.get_element_by_id("simuverse_container") {
                        let rect = container.get_bounding_client_rect();
                        w = rect.width() as f32;
                        h = rect.height() as f32;
                        let _ = container.append_child(&web_sys::Element::from(canvas));
                    } else {
                        doc.body()
                            .map(|body| body.append_child(&web_sys::Element::from(canvas)));
                    }
                    // winit 0.29 开始，通过 request_inner_size, canvas.set_width 都无法设置 canvas 的大小
                    let canvas = window.canvas().unwrap();
                    canvas.set_width((w * scale_factor) as u32);
                    canvas.set_height((h * scale_factor) as u32);
                    canvas.style().set_css_text(
                        &(canvas.style().css_text()
                            + &format!("background-color: black; display: block; margin: 20px auto; width: {}px; max-width: {}px; height: {}px; max-height: {}px", w, w, h, h)),
                    );
                })
                .expect("Couldn't append canvas to document body.");
        };

        #[cfg(not(target_arch = "wasm32"))]
        {
            let app_surface = pollster::block_on(AppSurface::new(window.clone()));
            let mut simu_app = pollster::block_on(SimuverseApp::new(app_surface, &window));
            simu_app.start();
            simu_app.app_surface.request_redraw();

            self.app = Rc::new(Mutex::new(Some(simu_app)));
        }
        #[cfg(target_arch = "wasm32")]
        {
            let app = self.app.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let app_surface = AppSurface::new(window.clone()).await;
                let mut simu_app = SimuverseApp::new(app_surface, &window).await;
                simu_app.start();
                simu_app.app_surface.request_redraw();

                let mut app = app.lock().unwrap();
                *app = Some(simu_app);
            });
        }
    }
}

impl ApplicationHandler for SimuverseAppHandler {
    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        self.wait_cancelled = match cause {
            StartCause::WaitCancelled { .. } => self.mode == Mode::WaitUntil,
            StartCause::Init => false,
            _ => false,
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.app.as_ref().lock().unwrap().is_some() {
            return;
        }

        let window_attributes = Window::default_attributes().with_title("simuverse");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.create_app(window);
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let mut app = self.app.lock().unwrap();
        if app.as_ref().is_none() {
            return;
        }

        app.as_mut().unwrap().on_ui_event(&event);

        match event {
            WindowEvent::CloseRequested => {
                self.close_requested = true;
            }
            WindowEvent::Resized(physical_size) => {
                if physical_size.width == 0 || physical_size.height == 0 {
                    // 处理最小化窗口的事件
                    println!("Window minimized!");
                } else {
                    app.as_mut().unwrap().resize(&physical_size);
                    #[cfg(target_arch = "wasm32")]
                    try_call_canvas_resize_completed();
                }
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
                ..
            } => {
                app.as_mut().unwrap().mouse_input(&state, &button);
                if button == MouseButton::Left && state == ElementState::Pressed {
                    let point = self.last_touch_point;
                    app.as_mut().unwrap().on_click(point);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                app.as_mut().unwrap().cursor_moved(position);

                let point = Vec2::new(position.x as f32, position.y as f32);
                app.as_mut().unwrap().touch_move(point);
                self.last_touch_point = point;
            }
            WindowEvent::MouseWheel { delta, phase, .. } => {
                app.as_mut().unwrap().mouse_wheel(&delta, &phase)
            }
            WindowEvent::RedrawRequested => {
                app.as_mut().unwrap().app_surface.pre_present_notify();

                app.as_mut().unwrap().render();

                app.as_mut().unwrap().app_surface.request_redraw();
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        match self.mode {
            Mode::Wait => event_loop.set_control_flow(ControlFlow::Wait),
            Mode::WaitUntil => {
                if !self.wait_cancelled {
                    event_loop
                        .set_control_flow(ControlFlow::WaitUntil(time::Instant::now() + WAIT_TIME));
                }
            }
            Mode::Poll => {
                thread::sleep(POLL_SLEEP_TIME);
                event_loop.set_control_flow(ControlFlow::Poll);
            }
        };

        if self.close_requested {
            event_loop.exit();
        }
    }
}
