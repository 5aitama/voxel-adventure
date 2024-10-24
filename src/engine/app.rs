use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use super::renderer::Renderer;

const WINDOW_TITLE: &str = "Voxel Engine";

#[derive(Default)]
pub struct App<'window> {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer<'window>>,
}

impl<'window> ApplicationHandler for App<'window> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attribute =
            winit::window::Window::default_attributes().with_inner_size(LogicalSize::new(800, 600));

        match event_loop.create_window(attribute) {
            Ok(window) => {
                window.set_title(WINDOW_TITLE);

                let window = Arc::new(window);
                self.window = Some(window.clone());
                self.renderer = Some(Renderer::new(window.clone()));
                self.window.as_ref().unwrap().request_redraw();
            }
            Err(err) => eprintln!("Failed to create a window: {:?}", err),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(new_size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    // renderer.resize(new_size);
                }
            }

            WindowEvent::RedrawRequested => {
                if let Some(window) = self.window.as_ref() {
                    if window.id() == window_id {
                        if let Some(renderer) = self.renderer.as_mut() {
                            if renderer.render().is_err() {
                                event_loop.exit();
                            }
                        }
                        window.request_redraw();
                    }
                }
            }

            WindowEvent::CloseRequested => {
                if let Some(window) = self.window.as_ref() {
                    if window.id() == window_id {
                        event_loop.exit();
                    }
                }
            }
            _ => {}
        }
    }
}
