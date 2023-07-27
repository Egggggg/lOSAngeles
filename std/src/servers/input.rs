use abi::{Status, ipc::Message};

pub use abi::input::*;

use crate::{ipc::{send_message, notify}, await_notif, await_notif_from};

pub fn subscribe() -> SubscribeStatus {
    let data0 = [
        Command::subscribe as u8,
        0, 0, 0, 0, 0, 0, 0
    ];
    let data0 = u64::from_be_bytes(data0);

    let status = notify(Message {
        pid: 3,
        data0,
        ..Default::default()
    });

    if status.is_err() {
        panic!("Couldn't send message to input server: {:?}", status);
    }

    let _ = await_notif_from(1, 0);

    SubscribeStatus::Success
}