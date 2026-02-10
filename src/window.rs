use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::PhysicalPosition,
    event::{self, WindowEvent},
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::wgpu::WgpuState;

#[derive(Default)]
pub struct App<'a> {
    state: Option<WgpuState<'a>>,
    window: Option<Arc<Window>>,

    cursor_position: Option<PhysicalPosition<f64>>,
}

impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = event_loop
                .create_window(Window::default_attributes().with_title("Mandelbrot"))
                .expect("Unable to create window");
            let window = Arc::new(window);

            self.window = Some(window.clone());

            let wgpu_state = pollster::block_on(WgpuState::new(window.clone()));
            self.state = Some(wgpu_state);

            println!("Window created.");
            println!("Controls:");
            println!("  - Scroll: Zoom in/out");

            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Close requested, exiting.");
                event_loop.exit();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = Some(position);
            }
            // Handle Scrolling (Zoom)
            WindowEvent::MouseWheel { delta, .. } => {
                if let Some(state) = &mut self.state {
                    let zoom_factor = 1.05; // 5% zoom per tick

                    let old_zoom = state.uniform_data.zoom;
                    let mut new_zoom = old_zoom;

                    match delta {
                        event::MouseScrollDelta::LineDelta(_, y) => {
                            if y > 0.0 {
                                new_zoom *= zoom_factor;
                            } else {
                                new_zoom /= zoom_factor;
                            }
                        }
                        event::MouseScrollDelta::PixelDelta(pos) => {
                            if pos.y > 0.0 {
                                new_zoom *= zoom_factor;
                            } else {
                                new_zoom /= zoom_factor;
                            }
                        }
                    }

                    if let Some(pos) = self.cursor_position {
                        let width = state.config.width as f32;
                        let height = state.config.height as f32;
                        let aspect = width / height;

                        // Calculate Mouse Position in NDC (-1.0 to 1.0)
                        let ndc_x = (pos.x as f32 / width) * 2.0 - 1.0;
                        let ndc_y = 1.0 - (pos.y as f32 / height) * 2.0;

                        // Calculate the "Mouse Vector" (Distance from center in fractal space)
                        let mouse_vec_x = ndc_x * aspect;
                        let mouse_vec_y = ndc_y;

                        // Calculate how much the world "shrank" relative to the center
                        // Formula: (1/OldZoom - 1/NewZoom) tells us the world-space shift required
                        let zoom_diff = (1.0 / old_zoom) - (1.0 / new_zoom);

                        // Move the center to compensate
                        state.uniform_data.center[0] += mouse_vec_x * zoom_diff;
                        state.uniform_data.center[1] += mouse_vec_y * zoom_diff;
                    }

                    state.uniform_data.zoom = new_zoom;

                    println!("Zoom: {:.2e}", new_zoom);
                    self.window.as_ref().unwrap().request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.state {
                    state.update();
                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(state) = &mut self.state {
                    state.resize(physical_size);
                }
            }
            _ => {}
        }
    }
}
