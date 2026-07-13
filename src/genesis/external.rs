use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

pub trait External {
    fn output_pixel(&mut self, pixel: u16, x: u16, y: u16);

    fn frame_complete(&mut self);

    fn open(&self) -> bool;

    fn button_array(&self) -> &[bool; 12];
}

pub struct RealExternal {
    buffer: Vec<u32>,
    window: Window,
    keys: [bool; 12] // A B C X Y Z Up Down Left Right Start Mode
}
impl RealExternal {
    pub fn new() -> RealExternal {
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
        RealExternal {
            buffer: vec![0; WIDTH * HEIGHT],
            window: window,
            keys: [false; 12]
        }
    }
}
impl External for RealExternal {
    fn output_pixel(&mut self, pixel: u16, x: u16, y: u16) {
        let red = ((pixel as u32) & 0x000F) << 20;
        let green = ((pixel as u32) & 0x00F0) << 8;
        let blue = ((pixel as u32) & 0x0F00) >> 4;
        self.buffer[(x as usize) + ((y as usize) * WIDTH)] = red + green + blue;
    }

    fn frame_complete(&mut self) {
        if self.window.is_open() {
            self.window.update_with_buffer(&self.buffer, WIDTH, HEIGHT).unwrap();
            self.keys = [self.window.is_key_down(Key::Z), self.window.is_key_down(Key::X),
                self.window.is_key_down(Key::C), self.window.is_key_down(Key::A),
                self.window.is_key_down(Key::S), self.window.is_key_down(Key::D),
                self.window.is_key_down(Key::Up), self.window.is_key_down(Key::Down),
                self.window.is_key_down(Key::Left), self.window.is_key_down(Key::Right),
                self.window.is_key_down(Key::Enter), self.window.is_key_down(Key::Space)];
        }
    }

    fn open(&self) -> bool {
        self.window.is_open()
    }

    fn button_array(&self) -> &[bool; 12] {
        &self.keys
    }
}

pub struct DummyExternal {

}
impl DummyExternal {
    pub fn new() -> DummyExternal {
        DummyExternal {

        }
    }
}
impl External for DummyExternal {
    fn output_pixel(&mut self, _pixel: u16, _x: u16, _y: u16) {

    }

    fn frame_complete(&mut self) {

    }

    fn open(&self) -> bool {
        true
    }

    fn button_array(&self) -> &[bool; 12] {
        &[false; 12]
    }
}