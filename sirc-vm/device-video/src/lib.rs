use std::{any::Any, borrow::BorrowMut, time::Duration};

use log::debug;
use peripheral_bus::{
    device::BusAssertions, device::Device, memory_mapped_device::MemoryMappedDevice,
};
use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    platform::pump_events::{EventLoopExtPumpEvents, PumpStatus},
    window::{Window, WindowId},
};

// Some reference: https://www.raphnet.net/divers/retro_challenge_2019_03/qsnesdoc.html#Reg2115

// 64kb = 32kw
const VRAM_SIZE: usize = 32_000;
// PAL can have a higher resolution but to keep things simple
// the renderer size can be static and PAL can have black bars
const WIDTH_PIXELS: usize = 256;
const HEIGHT_PIXELS: usize = 224;

const BOX_SIZE: i16 = 64;
/// Representation of the application state. In this example, a box will bounce around the screen.
struct World {
    box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,
}

impl World {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        Self {
            box_x: 24,
            box_y: 16,
            velocity_x: 1,
            velocity_y: 1,
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        if self.box_x <= 0 || self.box_x + BOX_SIZE > WIDTH_PIXELS as i16 {
            self.velocity_x *= -1;
        }
        if self.box_y <= 0 || self.box_y + BOX_SIZE > HEIGHT_PIXELS as i16 {
            self.velocity_y *= -1;
        }

        self.box_x += self.velocity_x;
        self.box_y += self.velocity_y;
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: [`pixels::wgpu::TextureFormat::Rgba8UnormSrgb`]
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH_PIXELS as usize) as i16;
            let y = (i / WIDTH_PIXELS as usize) as i16;

            let inside_the_box = x >= self.box_x
                && x < self.box_x + BOX_SIZE
                && y >= self.box_y
                && y < self.box_y + BOX_SIZE;

            let rgba = if inside_the_box {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x48, 0xb2, 0xe8, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }
}

struct RendererState {
    pixels: Pixels,
    world: World,
    time: f64,
}

struct VideoDeviceApplication {
    renderer_state: Option<RendererState>,
}

pub struct VideoDevice {
    // 3rd Party
    event_loop: EventLoop<()>,
    application: VideoDeviceApplication,

    // Native
    vram: Vec<u16>,
}

impl ApplicationHandler for VideoDeviceApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let size = LogicalSize::new(WIDTH_PIXELS as f64, HEIGHT_PIXELS as f64);
        let scaled_size = LogicalSize::new(WIDTH_PIXELS as f64 * 3.0, HEIGHT_PIXELS as f64 * 3.0);
        let window_attributes = Window::default_attributes()
            .with_title("SIRC")
            .with_inner_size(scaled_size)
            .with_min_inner_size(size);
        let window = event_loop.create_window(window_attributes).unwrap();
        let window_size = window.inner_size();
        self.renderer_state = Some(RendererState {
            world: World::new(),
            time: 0.0,
            pixels: {
                let surface_texture =
                    SurfaceTexture::new(window_size.width, window_size.height, &window);
                Pixels::new(WIDTH_PIXELS as u32, HEIGHT_PIXELS as u32, surface_texture).unwrap()
            },
        })
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        println!("{event:?}");

        if let Some(r) = self.renderer_state.borrow_mut() {
            r.world.draw(r.pixels.frame_mut());
            r.world.update();
        }
        // let window = match self.window.as_ref() {
        //     Some(window) => window,
        //     None => return,
        // };

        // match event {
        //     WindowEvent::CloseRequested => event_loop.exit(),
        //     WindowEvent::RedrawRequested => {
        //         window.request_redraw();
        //     }
        //     _ => (),
        // }
    }
}

#[must_use]
pub fn new_video_device() -> VideoDevice {
    #[cfg(any(ios_platform, web_platform, orbital_platform))]
    panic!("This platform does not support pump events");

    let event_loop = EventLoop::new().unwrap();

    VideoDevice {
        event_loop,
        application: VideoDeviceApplication {
            renderer_state: None,
        },
        vram: vec![0; VRAM_SIZE],
    }
}

impl Device for VideoDevice {
    fn poll(&mut self, bus_assertions: BusAssertions, selected: bool) -> BusAssertions {
        let timeout = Some(Duration::ZERO);
        let status = self
            .event_loop
            .pump_app_events(timeout, &mut self.application);

        if let PumpStatus::Exit(exit_code) = status {
            panic!("Video device exited with code [{}]", exit_code);
        }

        self.perform_bus_io(bus_assertions, selected)
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

impl MemoryMappedDevice for VideoDevice {
    fn read_address(&self, address: u32) -> u16 {
        debug!("Reading from address 0x{address:X}");
        match address {
            // First FF addresses are control registers
            // TODO: Actually implement some control registers
            0x0000..=0x00FF => 0x0,
            // After that range
            _ => self.vram[(address as usize) - 0x00FF],
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn write_address(&mut self, address: u32, value: u16) {
        debug!("Writing 0x{value:X} to address 0x{address:X}");
        match address {
            // First FF addresses are control registers
            // TODO: Actually implement some control registers
            0x0000..=0x00FF => {
                // TODO
            }
            // After that range
            _ => self.vram[(address as usize) - 0x00FF] = value,
        }
    }
}
