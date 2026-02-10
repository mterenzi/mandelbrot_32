use winit::event_loop::{ControlFlow, EventLoop};

use crate::window::App;

mod primitives;
mod wgpu;
mod window;

fn main() {
    env_logger::init();

    pollster::block_on(run());
}

async fn run() {
    // Setup the event loop
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    // Create our app state
    let mut app = App::default();

    event_loop.run_app(&mut app).unwrap();
}
