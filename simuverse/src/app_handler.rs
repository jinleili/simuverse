use crate::SimuverseApp;
use alloc::rc::Rc;
use alloc::sync::Arc;
use glam::Vec2;
use parking_lot::Mutex;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

pub fn run() -> Result<(), impl std::error::Error> {
    let events_loop = EventLoop::new().unwrap();
    let mut handler = SimuverseAppHandler::default();
    events_loop.run_app(&mut handler)
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = Function, js_name = "prototype.call.call")]
    fn call_catch(this: &JsValue) -> Result<(), JsValue>;
}

#[derive(Default)]
struct SimuverseAppHandler {
    last_touch_point: Vec2,
    window: Option<Arc<Window>>,
    app: Rc<Mutex<Option<SimuverseApp>>>,

    /// 错失的窗口大小变化
    ///
    /// # NOTE：
    /// 在 web 端，app 的初始化是异步的，当收到 resized 事件时，初始化可能还没有完成从而错过窗口 resized 事件，
    /// 当 app 初始化完成后会调用 `set_window_resized` 方法来补上错失的窗口大小变化事件。
    #[allow(dead_code)]
    missed_resize: Rc<Mutex<Option<PhysicalSize<u32>>>>,

    /// 错失的请求重绘事件
    ///
    /// # NOTE：
    /// 在 web 端，app 的初始化是异步的，当收到 redraw 事件时，初始化可能还没有完成从而错过请求重绘事件，
    /// 当 app 初始化完成后会调用 `request_redraw` 方法来补上错失的请求重绘事件。
    #[allow(dead_code)]
    missed_request_redraw: Rc<Mutex<bool>>,
}

impl SimuverseAppHandler {
    fn create_app(&mut self, window: Arc<Window>) {
        if cfg!(not(target_arch = "wasm32")) {
            // 计算一个默认显示高度
            let height = (750.0 * window.scale_factor()) as u32;
            let width = (height as f32 * 1.6) as u32;
            let _ = window.request_inner_size(PhysicalSize::new(width, height));
        }

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;

            // 将 canvas 添加到当前网页中
            let canvas = window.canvas().unwrap();
            web_sys::window()
                .and_then(|win| win.document())
                .map(|doc| {
                    let _ = canvas.set_attribute("id", "simuverse_app");
                    match doc.get_element_by_id("simuverse_container") {
                        Some(dst) => {
                            let _ = dst.append_child(canvas.as_ref());
                        }
                        None => {
                            let container = doc.create_element("div").unwrap();
                            let _ = container.set_attribute("id", "simuverse_container");
                            let _ = container.append_child(canvas.as_ref());

                            doc.body().map(|body| body.append_child(container.as_ref()));
                        }
                    };
                })
                .expect("无法将 canvas 添加到当前网页中");

            // 确保画布可以获得焦点
            // https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/tabindex
            canvas.set_tab_index(0);

            // 设置画布获得焦点时不显示高亮轮廓
            let style = canvas.style();
            style.set_property("outline", "none").unwrap();
            canvas.focus().expect("画布无法获取焦点");
        };

        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let app = self.app.clone();
                let missed_resize = self.missed_resize.clone();
                let missed_request_redraw = self.missed_request_redraw.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    let window_cloned = window.clone();

                    let simu_app = SimuverseApp::new(window).await;
                    let mut app = app.lock();
                    *app = Some(simu_app);

                    if let Some(resize) = *missed_resize.lock() {
                        app.as_mut().unwrap().set_window_resized(resize);
                    }

                    if *missed_request_redraw.lock() {
                        window_cloned.request_redraw();
                    }
                });
            } else {
                let simu_app = pollster::block_on(SimuverseApp::new(window));
                self.app.lock().replace(simu_app);
            }
        }
    }

    /// 在提交渲染之前通知窗口系统。
    fn pre_present_notify(&self) {
        if let Some(window) = self.window.as_ref() {
            window.pre_present_notify();
        }
    }

    /// 请求重绘    
    fn request_redraw(&self) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

impl ApplicationHandler for SimuverseAppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.app.as_ref().lock().is_some() {
            return;
        }

        let window_attributes = Window::default_attributes().with_title("simuverse");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.window = Some(window.clone());
        self.create_app(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let mut app = self.app.lock();
        if app.as_ref().is_none() {
            // 如果 app 还没有初始化完成，则记录错失的窗口事件
            match event {
                WindowEvent::Resized(physical_size) => {
                    if physical_size.width > 0 && physical_size.height > 0 {
                        let mut missed_resize = self.missed_resize.lock();
                        *missed_resize = Some(physical_size);
                    }
                }
                WindowEvent::RedrawRequested => {
                    let mut missed_request_redraw = self.missed_request_redraw.lock();
                    *missed_request_redraw = true;
                }
                _ => (),
            }
            return;
        }

        let app = app.as_mut().unwrap();
        app.on_ui_event(&event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(physical_size) => {
                if physical_size.width == 0 || physical_size.height == 0 {
                    // 处理最小化窗口的事件
                    log::info!("Window minimized!");
                } else {
                    app.set_window_resized(physical_size);
                }
            }
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button,
                ..
            } => {
                app.mouse_input(&state, &button);
                if button == MouseButton::Left && state == ElementState::Pressed {
                    let point = self.last_touch_point;
                    app.on_click(point);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                app.cursor_moved(position);

                let point = Vec2::new(position.x as f32, position.y as f32);
                app.touch_move(point);
                self.last_touch_point = point;
            }
            WindowEvent::MouseWheel { delta, phase, .. } => app.mouse_wheel(&delta, &phase),
            WindowEvent::RedrawRequested => {
                self.pre_present_notify();

                app.render();

                self.request_redraw();
            }
            _ => (),
        }
    }
}
