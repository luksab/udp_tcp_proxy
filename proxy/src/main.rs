#![feature(slice_as_chunks)]
use bus::Bus;
use bus::BusReader;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::net::UdpSocket;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Instant;

const BUFFER_SIZE: usize = 1024;

fn upd_stuff(sender: Arc<Mutex<Bus<Vec<u8>>>>) {
    let udp_socket = UdpSocket::bind("127.0.0.1:34254").unwrap();
    let mut buf = [0; 1024];
    let mut total_buffer = [0; BUFFER_SIZE];
    let mut current_buffer_index = 0;
    let mut perf_counter = 0;
    let mut perf_clock = Instant::now();
    loop {
        match udp_socket.recv_from(&mut buf) {
            Ok((amt, _src)) => {
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
                    // let mut vecu32 = Vec::with_capacity(BUFFER_SIZE / 4);
                    // for chunk in total_buffer.as_chunks().0 {
                    //     let u32: u32 = u32::from_le_bytes(*chunk);
                    //     vecu32.push(u32);
                    // }
                    sender.lock().unwrap().broadcast(Vec::from(total_buffer));
                    current_buffer_index = 0;

                    perf_counter += 1;
                    if perf_clock.elapsed().as_secs() > 1 {
                        println!(
                            "received {} packets aka {} MB",
                            perf_counter,
                            perf_counter * BUFFER_SIZE / 1024 / 1024
                        );
                        perf_counter = 0;
                        perf_clock = Instant::now();
                    }
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
    let bus = Arc::new(Mutex::new(Bus::new(2)));

    let tcp_listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    // let tcp_pool = ThreadPool::new(4, bus.clone());

    let udp_thread = {
        let bus = bus.clone();
        thread::spawn(move || upd_stuff(bus))
    };

    // rx.recv().expect("Could not receive from channel.");

    for stream in tcp_listener.incoming() {
        println!("new connection");
        // let rx = bus.add_rx();
        let stream = stream.unwrap();

        println!("spawning thread");
        let rx = bus.lock().unwrap().add_rx();
        thread::spawn(move || {
            handle_connection(stream, rx);
        });
        // tcp_pool.execute(move |rx: &mut BusReader<Vec<u8>>| {
        //     handle_connection(stream, rx);
        // });
    }

    udp_thread.join().unwrap();
}

fn handle_connection(mut stream: TcpStream, mut rx: BusReader<Vec<u8>>) {
    // let mut buffer = [0; 1024];
    // stream.read(&mut buffer).unwrap();

    // let get = b"GET / HTTP/1.1\r\n";
    // let sleep = b"GET /sleep HTTP/1.1\r\n";

    // let (status_line, filename) = if buffer.starts_with(get) {
    //     ("HTTP/1.1 200 OK", "hello.html")
    // } else if buffer.starts_with(sleep) {
    //     thread::sleep(Duration::from_secs(5));
    //     ("HTTP/1.1 200 OK", "hello.html")
    // } else {
    //     ("HTTP/1.1 404 NOT FOUND", "404.html")
    // };
    loop {
        let vec = match rx.recv() {
            Ok(v) => v,
            Err(_) => break,
        };

        // println!("Received {} bytes", vec.len());

        let response = format!("{:?}\n", vec);

        match stream.write(response.as_bytes()) {
            Ok(_) => {}
            Err(_) => break,
        };
        // match stream.write(&vec) {
        //     Ok(_) => {}
        //     Err(_) => break,
        // };
        match stream.flush() {
            Ok(_) => {}
            Err(_) => break,
        };
    }
    println!("Client disconnected.");
}
