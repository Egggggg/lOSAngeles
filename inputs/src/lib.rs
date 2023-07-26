#![no_std]

use std::{Status, InvalidStatusCode};

use num_derive::FromPrimitive;

extern crate alloc;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
#[allow(non_camel_case_types)]
pub enum Command {
    publish = 0x00,
    subscribe = 0x10,
}

#[derive(Clone, Copy, Debug)]
pub struct InvalidCommand;

impl TryFrom<u64> for Command {
    type Error = InvalidCommand;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::publish),
            0x10 => Ok(Self::subscribe),
            _ => Err(InvalidCommand),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u64)]
pub enum PublishStatus {
    Success = 0,
    IncomingKey = 1,
    MissingPermissions = 10,
    InvalidKey = 11,
}

impl TryFrom<u64> for PublishStatus {
    type Error = InvalidStatusCode;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Success),
            1 => Ok(Self::IncomingKey),
            10 => Ok(Self::MissingPermissions),
            11 => Ok(Self::InvalidKey),
            _ => Err(InvalidStatusCode),
        }
    }
}

impl From<PublishStatus> for u8 {
    fn from(value: PublishStatus) -> Self {
        value as u8
    }
}

impl Status for PublishStatus {}

// Taken from pc_keyboard crate under the MIT license so I can add this top line
#[derive(FromPrimitive)]
#[derive(Debug, PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
#[repr(u8)]
pub enum KeyCode {
    // ========= Row 1 (the F-keys) =========
    /// Top Left of the Keyboard
    Escape,
    /// Function Key F1
    F1,
    /// Function Key F2
    F2,
    /// Function Key F3
    F3,
    /// Function Key F4
    F4,
    /// Function Key F5
    F5,
    /// Function Key F6
    F6,
    /// Function Key F7
    F7,
    /// Function Key F8
    F8,
    /// Function Key F9
    F9,
    /// Function Key F10
    F10,
    /// Function Key F11
    F11,
    /// Function Key F12
    F12,

    /// The Print Screen Key
    PrintScreen,
    /// The Sys Req key (you get this keycode with Alt + PrintScreen)
    SysRq,
    /// The Scroll Lock key
    ScrollLock,
    /// The Pause/Break key
    PauseBreak,

    // ========= Row 2 (the numbers) =========
    /// Symbol key to the left of `Key1`
    Oem8,
    /// Number Line, Digit 1
    Key1,
    /// Number Line, Digit 2
    Key2,
    /// Number Line, Digit 3
    Key3,
    /// Number Line, Digit 4
    Key4,
    /// Number Line, Digit 5
    Key5,
    /// Number Line, Digit 6
    Key6,
    /// Number Line, Digit 7
    Key7,
    /// Number Line, Digit 8
    Key8,
    /// Number Line, Digit 9
    Key9,
    /// Number Line, Digit 0
    Key0,
    /// US Minus/Underscore Key (right of 'Key0')
    OemMinus,
    /// US Equals/Plus Key (right of 'OemMinus')
    OemPlus,
    /// Backspace
    Backspace,

    /// Top Left of the Extended Block
    Insert,
    /// Top Middle of the Extended Block
    Home,
    /// Top Right of the Extended Block
    PageUp,

    /// The Num Lock key
    NumpadLock,
    /// The Numpad Divide (or Slash) key
    NumpadDivide,
    /// The Numpad Multiple (or Star) key
    NumpadMultiply,
    /// The Numpad Subtract (or Minus) key
    NumpadSubtract,

    // ========= Row 3 (QWERTY) =========
    /// The Tab Key
    Tab,
    /// Letters, Top Row #1
    Q,
    /// Letters, Top Row #2
    W,
    /// Letters, Top Row #3
    E,
    /// Letters, Top Row #4
    R,
    /// Letters, Top Row #5
    T,
    /// Letters, Top Row #6
    Y,
    /// Letters, Top Row #7
    U,
    /// Letters, Top Row #8
    I,
    /// Letters, Top Row #9
    O,
    /// Letters, Top Row #10
    P,
    /// US ANSI Left-Square-Bracket key
    Oem4,
    /// US ANSI Right-Square-Bracket key
    Oem6,
    /// US ANSI Backslash Key / UK ISO Backslash Key
    Oem5,
    /// The UK/ISO Hash/Tilde key (ISO layout only)
    Oem7,

    /// The Delete key - bottom Left of the Extended Block
    Delete,
    /// The End key - bottom Middle of the Extended Block
    End,
    /// The Page Down key - -bottom Right of the Extended Block
    PageDown,

    /// The Numpad 7/Home key
    Numpad7,
    /// The Numpad 8/Up Arrow key
    Numpad8,
    /// The Numpad 9/Page Up key
    Numpad9,
    /// The Numpad Add/Plus key
    NumpadAdd,

    // ========= Row 4 (ASDF) =========
    /// Caps Lock
    CapsLock,
    /// Letters, Middle Row #1
    A,
    /// Letters, Middle Row #2
    S,
    /// Letters, Middle Row #3
    D,
    /// Letters, Middle Row #4
    F,
    /// Letters, Middle Row #5
    G,
    /// Letters, Middle Row #6
    H,
    /// Letters, Middle Row #7
    J,
    /// Letters, Middle Row #8
    K,
    /// Letters, Middle Row #9
    L,
    /// The US ANSI Semicolon/Colon key
    Oem1,
    /// The US ANSI Single-Quote/At key
    Oem3,

    /// The Return Key
    Return,

    /// The Numpad 4/Left Arrow key
    Numpad4,
    /// The Numpad 5 Key
    Numpad5,
    /// The Numpad 6/Right Arrow key
    Numpad6,

    // ========= Row 5 (ZXCV) =========
    /// Left Shift
    LShift,
    /// Letters, Bottom Row #1
    Z,
    /// Letters, Bottom Row #2
    X,
    /// Letters, Bottom Row #3
    C,
    /// Letters, Bottom Row #4
    V,
    /// Letters, Bottom Row #5
    B,
    /// Letters, Bottom Row #6
    N,
    /// Letters, Bottom Row #7
    M,
    /// US ANSI `,<` key
    OemComma,
    /// US ANSI `.>` Key
    OemPeriod,
    /// US ANSI `/?` Key
    Oem2,
    /// Right Shift
    RShift,

    /// The up-arrow in the inverted-T
    ArrowUp,

    /// Numpad 1/End Key
    Numpad1,
    /// Numpad 2/Arrow Down Key
    Numpad2,
    /// Numpad 3/Page Down Key
    Numpad3,
    /// Numpad Enter
    NumpadEnter,

    // ========= Row 6 (modifers and space bar) =========
    /// The left-hand Control key
    LControl,
    /// The left-hand 'Windows' key
    LWin,
    /// The left-hand Alt key
    LAlt,
    /// The Space Bar
    Spacebar,
    /// The right-hand AltGr key
    RAltGr,
    /// The right-hand Win key
    RWin,
    /// The 'Apps' key (aka 'Menu' or 'Right-Click')
    Apps,
    /// The right-hand Control key
    RControl,

    /// The left-arrow in the inverted-T
    ArrowLeft,
    /// The down-arrow in the inverted-T
    ArrowDown,
    /// The right-arrow in the inverted-T
    ArrowRight,

    /// The Numpad 0/Insert Key
    Numpad0,
    /// The Numppad Period/Delete Key
    NumpadPeriod,

    // ========= JIS 109-key extra keys =========
    /// Extra JIS key (0x7B)
    Oem9,
    /// Extra JIS key (0x79)
    Oem10,
    /// Extra JIS key (0x70)
    Oem11,
    /// Extra JIS symbol key (0x73)
    Oem12,
    /// Extra JIS symbol key (0x7D)
    Oem13,

    // ========= Extra Keys =========
    /// Multi-media keys - Previous Track
    PrevTrack,
    /// Multi-media keys - Next Track
    NextTrack,
    /// Multi-media keys - Volume Mute Toggle
    Mute,
    /// Multi-media keys - Open Calculator
    Calculator,
    /// Multi-media keys - Play
    Play,
    /// Multi-media keys - Stop
    Stop,
    /// Multi-media keys - Increase Volume
    VolumeDown,
    /// Multi-media keys - Decrease Volume
    VolumeUp,
    /// Multi-media keys - Open Browser
    WWWHome,
    /// Sent when the keyboard boots
    PowerOnTestOk,
    /// Sent by the keyboard when too many keys are pressed
    TooManyKeys,
    /// Used as a 'hidden' Right Control Key (Pause = RControl2 + Num Lock)
    RControl2,
    /// Used as a 'hidden' Right Alt Key (Print Screen = RAlt2 + PrntScr)
    RAlt2,
}