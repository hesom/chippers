use chippers::{chip8::Chip8, frontend::App};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    let mut chip8 = Chip8::new();
    chip8.load_rom("./roms/space_invaders.ch8");

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new(&mut chip8, 16);

    event_loop.run_app(&mut app).unwrap();
}
