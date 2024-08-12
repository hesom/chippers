use pixels::{Pixels, SurfaceTexture};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use crate::chip8::Chip8;

const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;

pub struct App<'a> {
    pub window: Option<Window>,
    pub pixels: Option<Pixels>,
    pub chip8: &'a mut Chip8,
    scale: u32,
}

impl<'a> App<'a> {
    pub fn new(chip8: &'a mut Chip8, scale: u32) -> Self {
        Self {
            window: None,
            pixels: None,
            chip8,
            scale,
        }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = {
            let size = LogicalSize::new((self.scale * WIDTH) as f64, (self.scale * HEIGHT) as f64);
            Some(
                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title("Chip8")
                            .with_inner_size(size)
                            .with_min_inner_size(size),
                    )
                    .unwrap(),
            )
        };
        self.pixels = {
            let window_size = self.window.as_ref().unwrap().inner_size();
            let surface_texture = SurfaceTexture::new(
                window_size.width,
                window_size.height,
                self.window.as_ref().unwrap(),
            );
            Some(
                Pixels::new(self.scale * WIDTH, self.scale * HEIGHT, surface_texture)
                    .expect("Pixel surface should be created"),
            )
        };
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: ()) {}

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let window = self.window.as_mut().unwrap();
        let pixels = self.pixels.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.chip8.cycle();
                if self.chip8.draw_flag {
                    self.chip8.draw_flag = false;

                    let framebuf = pixels.frame_mut();
                    for (i, pixel) in framebuf.chunks_exact_mut(4).enumerate() {
                        let x = (i % (self.scale * WIDTH) as usize) / self.scale as usize;
                        let y = (i / (self.scale * WIDTH) as usize) / self.scale as usize;

                        let colored = self.chip8.gfx[x + y * 64] == 1;

                        let rgba = if colored {
                            [0xFF, 0xFF, 0xFF, 0xFF]
                        } else {
                            [0x00, 0x00, 0x00, 0x00]
                        };

                        pixel.copy_from_slice(&rgba);
                    }
                    pixels.render().unwrap();
                }

                window.request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
            _ => (),
        }
    }
}
