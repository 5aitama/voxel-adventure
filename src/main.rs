// mod engine {
//     pub mod app;
//     pub mod passes {
//         pub mod vox_compute_pass;
//     }
//     pub mod renderer;
// }
mod engine;

fn main() {
    let ev_loop: winit::event_loop::EventLoop<()> = winit::event_loop::EventLoop::new().unwrap();
    let mut app = engine::app::App::default();
    ev_loop.run_app(&mut app).unwrap();
}
