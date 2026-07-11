use minifb::{Scale, ScaleMode, Window, WindowOptions};

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

pub trait Screen {
    fn output_pixel(&mut self, pixel: u16, x: u16, y: u16);

    fn frame_complete(&mut self);
}

pub struct DisplayScreen {
    buffer: Vec<u32>,
    window: Window,
}
impl DisplayScreen {
    pub fn new() -> DisplayScreen {
        let options = WindowOptions {
            borderless: false,
            title: true,
            resize: false,
            scale: Scale::X2,
            scale_mode: ScaleMode::Stretch,
            topmost: false,
            transparency: false,
            none: false,
        };
        let mut window = Window::new(
            "Genonysus",
            WIDTH,
            HEIGHT,
            options,
        )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });
        //window.set_target_fps(60);
        DisplayScreen {
            buffer: vec![0; WIDTH * HEIGHT],
            window: window,
        }
    }
}
impl Screen for DisplayScreen {
    fn output_pixel(&mut self, pixel: u16, x: u16, y: u16) {
        let red = ((pixel as u32) & 0x000F) << 20;
        let green = ((pixel as u32) & 0x00F0) << 8;
        let blue = ((pixel as u32) & 0x0F00) >> 4;
        self.buffer[(x as usize) + ((y as usize) * WIDTH)] = red + green + blue;
    }

    fn frame_complete(&mut self) {
        if self.window.is_open() {
            self.window.update_with_buffer(&self.buffer, WIDTH, HEIGHT).unwrap();
        }
    }
}

pub struct DummyScreen {

}
impl DummyScreen {
    pub fn new() -> DummyScreen {
        DummyScreen {

        }
    }
}
impl Screen for DummyScreen {
    fn output_pixel(&mut self, _pixel: u16, _x: u16, _y: u16) {

    }

    fn frame_complete(&mut self) {

    }
}