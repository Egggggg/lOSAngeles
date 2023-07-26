use pc_keyboard::{Keyboard, ScancodeSet, KeyboardLayout, KeyEvent};

pub fn decode<T: KeyboardLayout, S: ScancodeSet>(scancode: u8, keyboard: &mut Keyboard<T, S>) -> Option<KeyEvent> {
    if let Ok(key_event) = keyboard.add_byte(scancode) {
        key_event
    } else {
        None
    }
}