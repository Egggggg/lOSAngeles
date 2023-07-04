//! This program starts a server in process 1 and a client in process 2
//! The client requests a shared region of memory from the server, and the server requests it from the kernel
//! The client and server then enter a loop of of reading and writing to the shared memory
//! The server counts how many times it has received messages from the client telling it to continue
//! Once this count reaches 100, the server tells the client it can stop

#![no_std]
#![no_main]

use programs::{getpid, send, receive, Message, SendStatus, join_memshare, exit, println, create_memshare, sys_yield};

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

    create_memshare(1, 0, 0, &[2]);

    println!("1: Memshare created");

    send(Message {
        pid: 2,
        ..Default::default()
    });

    receive(&[2]);

    let ptr = 2048 as *const u8;

    println!("*ptr: {}", unsafe { *ptr });
    exit();
}

fn run_client() {
    println!("2: Client started");

    receive(&[1]);

    println!("2: Memshare ready, joining");

    join_memshare(1, 4096, 4096, &[]);

    let ptr = 6144 as *mut u8;

    unsafe { *ptr = 69 };

    send(Message {
        pid: 1,
        ..Default::default()
    });
    exit();
}