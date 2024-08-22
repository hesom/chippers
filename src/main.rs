use chippers::{chip8::Chip8, frontend::App};
use clap::Parser;
use std::path::PathBuf;
use winit::event_loop::{ControlFlow, EventLoop};

#[derive(Parser, Debug)]
struct Cli {
    rom_path: PathBuf,

    #[arg(short, long, default_value_t = 16)]
    scale: u32,
}

fn main() {
    let args = Cli::parse();

    let mut chip8 = Chip8::new();
    chip8.load_rom(args.rom_path);

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new(&mut chip8, args.scale);

    event_loop.run_app(&mut app).unwrap();
}
