use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

const BUFFER_SIZE: usize = 1024;

fn upd_stuff() {
    let udp_socket = UdpSocket::bind("127.0.0.1:34255").unwrap();
    let mut buf = [0; BUFFER_SIZE];
    let mut now = std::time::Instant::now();
    let mut counter = 0;
    loop {
        // for i in 0..BUFFER_SIZE {
        //     buf[i] = fastrand::u8(0..=255);
        // }
        udp_socket.send_to(&buf, "127.0.0.1:34254").unwrap();
        counter += 1;
        if now.elapsed() > Duration::from_secs(1) {
            now = std::time::Instant::now();
            println!("sent {} packets, aka {} MB", counter, counter * BUFFER_SIZE / 1024 / 1024);
            counter = 0;
        }
        // println!("Sent {} bytes", BUFFER_SIZE);
        // thread::sleep(Duration::from_millis(1000));
    }
}

fn main() {
    let udp_thread = thread::spawn(upd_stuff);
    udp_thread.join().unwrap();
}
