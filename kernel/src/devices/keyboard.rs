use pc_keyboard::{layouts::Us104Key, ScancodeSet1, DecodedKey};

const BUFFER_SIZE: usize = 32;

pub struct Keyboard {
    pub device: pc_keyboard::Keyboard<Us104Key, ScancodeSet1>,
    pub buffer: [Option<DecodedKey>; BUFFER_SIZE],
}

impl Keyboard {
    pub const fn new() -> Self {
        Self {
            device: pc_keyboard::Keyboard::new(ScancodeSet1::new(), Us104Key, pc_keyboard::HandleControl::Ignore),
            buffer: [None; BUFFER_SIZE],
        }
    }

    pub fn handle_key(&mut self, scancode: u8) {
        if let Ok(Some(key_event)) = self.device.add_byte(scancode) {
            if let Some(key) = self.device.process_keyevent(key_event) {
                self.buffer.rotate_right(1);
                self.buffer[0] = Some(key);
            }
        }
    }
}