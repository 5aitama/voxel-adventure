use renderer::app::App;

mod renderer;

fn main() {
    let ev_loop: winit::event_loop::EventLoop<()> = winit::event_loop::EventLoop::new().unwrap();
    let mut app = App::default();
    ev_loop.run_app(&mut app).unwrap();
}
