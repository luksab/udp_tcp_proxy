use std::io;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;

const BUFFER_SIZE: usize = 1024*32;

#[tokio::main]
async fn main() -> io::Result<()> {
    let sock = UdpSocket::bind("0.0.0.0:8080").await?;
    let r = Arc::new(sock);
    let atomic_counter = Arc::new(AtomicUsize::new(0));

    for _ in 0..1 {
        println!("new thread");
        let s = r.clone();
        let counter = atomic_counter.clone();
        tokio::spawn(async move {
            let mut buf = [0; BUFFER_SIZE];
            for _ in 0..BUFFER_SIZE {
                buf[0] = fastrand::u8(0..=255);
            }
            // let mut counter = 0;
            loop {
                // let (len, addr) = sock.recv_from(&mut buf).await?;
                // println!("{:?} bytes received from {:?}", len, addr);
                let _ = s.send_to(&buf, "127.0.0.1:34254").await.unwrap();
                // println!("{:?} bytes sent", len);

                counter.fetch_add(1, Ordering::Relaxed);
            }
        });
    }

    let mut now = std::time::Instant::now();
    loop {
        if now.elapsed() > Duration::from_secs(1) {
            println!(
                "sent {} packets, aka {} MB in {} ms",
                atomic_counter.load(Ordering::Relaxed),
                atomic_counter.load(Ordering::Relaxed) * BUFFER_SIZE / 1024 / 1024,
                now.elapsed().as_millis()
            );
            now = std::time::Instant::now();
            atomic_counter.store(0, Ordering::Relaxed);
        }
    }
}
