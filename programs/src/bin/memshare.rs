//! This program starts a server in process 1 and a client in process 2
//! The client requests a shared region of memory from the server, and the server requests it from the kernel
//! The client and server then enter a loop of of reading and writing to the shared memory
//! The server counts how many times it has received messages from the client telling it to continue
//! Once this count reaches 100, the server tells the client it can stop

#![no_std]
#![no_main]

use programs::{getpid, send, receive, Message, SendStatus, join_memshare, exit, println, create_memshare, sys_yield, CreateShareStatus, JoinShareStatus};

#[no_mangle]
pub unsafe extern "C" fn _start() {
    let pid = getpid();

    match pid {
        1 => run_server(),
        2 => run_client(),
        e => panic!("why god why ({})", e),
    }
}

fn run_server() {
    println!("1: Server started");
    
    let start = 0;
    let end = 16384;

    match create_memshare(1, start, end, &[2]) {
        CreateShareStatus::Success => {},
        s => panic!("1: Share failed: {:?}", s),
    }

    println!("1: Memshare created");

    send(Message {
        pid: 2,
        data0: start,
        data1: end,
        ..Default::default()
    });

    println!("1: Message sent");

    receive(&[2]);

    println!("1: Checking *ptr");

    let ptr = 2048 as *const u8;
    let target = 10240;

    println!("1: *ptr: {}", unsafe { *ptr });
    println!("1: Hey 2 look at ${:#04X} u16 style", target);

    let ptr = target as *mut u16;

    unsafe { *ptr = 2048 };

    send(Message {
        pid: 2,
        data0: target,
        ..Default::default()
    });

    println!("1: Exiting");

    exit();
}

fn run_client() {
    println!("2: Client started");

    let (_, msg) = receive(&[1]);

    println!("2: Memshare ready, joining");

    match join_memshare(1, msg.data0, msg.data1, &[]) {
        JoinShareStatus::Success => {},
        e => panic!("2: Share failed: {:?}", e),
    }

    println!("2: Memshare joined");

    let ptr = 2048 as *mut u8;

    unsafe { *ptr = 69 };

    println!("2: *ptr set");

    let ptr = 2048 as *const u8;

    println!("2: *ptr: {}", unsafe { *ptr });

    send(Message {
        pid: 1,
        ..Default::default()
    });

    let (_, msg) = receive(&[1]);
    let ptr = msg.data0 as *const u16;

    println!("2: Haha! It's {}", unsafe { *ptr });
    println!("2: Exiting");

    exit();
}