use abi::{Status, ipc::Message};

pub use abi::input::*;

use crate::{ipc::send_message, await_notif};

pub fn subscribe() -> SubscribeStatus {
    let data0 = [
        Command::subscribe as u8,
        0, 0, 0, 0, 0, 0, 0
    ];
    let data0 = u64::from_be_bytes(data0);

    let status = send_message(Message {
        pid: 3,
        data0,
        ..Default::default()
    });

    if status.is_err() {
        panic!("Couldn't send message to input server: {:?}", status);
    }

    let _ = await_notif(1, 0);

    SubscribeStatus::Success
}