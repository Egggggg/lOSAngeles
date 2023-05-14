mod keyboard;

use keyboard::Keyboard;

pub static KEYBOARD: spin::Mutex<Keyboard> = spin::Mutex::new(Keyboard::new());