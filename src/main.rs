#![feature(slice_as_chunks)]
use crossbeam_channel::{unbounded, Receiver, Sender};
use std::fs;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

use udp_tcp_proxy::ThreadPool;

const BUFFER_SIZE: usize = 32;

fn upd_stuff(sender: Sender<Vec<u32>>) {
    let udp_socket = UdpSocket::bind("127.0.0.1:34254").unwrap();
    let mut buf = [0; 1024];
    let mut total_buffer = [0; BUFFER_SIZE];
    let mut current_buffer_index = 0;
    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                let buf = &mut buf[..amt];
                let new_index = if amt + current_buffer_index > BUFFER_SIZE {
                    BUFFER_SIZE
                } else {
                    amt + current_buffer_index
                };
                total_buffer[current_buffer_index..new_index]
                    .copy_from_slice(&buf[..(new_index - current_buffer_index)]);

                current_buffer_index = new_index;
                if current_buffer_index == BUFFER_SIZE {
                    let mut vecu32 = Vec::with_capacity(BUFFER_SIZE / 4);
                    for chunk in total_buffer.as_chunks().0 {
                        let u32: u32 = u32::from_le_bytes(*chunk);
                        vecu32.push(u32);
                    }
                    sender.send(vecu32).unwrap();
                    current_buffer_index = 0;
                }

                // sender.send(()).unwrap();
                // println!("Received {} bytes from {}", amt, src);
                // println!("{}", String::from_utf8_lossy(buf));
            }
            Err(_) => todo!(),
        };
    }
}

fn main() {
    let (tx, rx) = unbounded();

    let udp_thread = thread::spawn(move || upd_stuff(tx));

    let tcp_listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let tcp_pool = ThreadPool::new(4);

    // rx.recv().expect("Could not receive from channel.");

    for stream in tcp_listener.incoming() {
        let rx = rx.clone();
        let stream = stream.unwrap();

        tcp_pool.execute(move || {
            handle_connection(stream, rx);
        });
    }

    udp_thread.join().unwrap();
}

fn handle_connection(mut stream: TcpStream, rx: Receiver<Vec<u32>>) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    println!("serving {}", filename);

    let vec = rx.recv().expect("Could not receive from channel.");

    let contents = format!("{:?}", vec); //fs::read_to_string(filename).unwrap();
                                         // let buf = vec.iter().map(|x| x.to_le_bytes()).flatten().collect::<Vec<u8>>();
                                         // let contents = String::from_utf8_lossy(&buf);

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
