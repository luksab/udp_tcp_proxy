#![feature(slice_as_chunks)]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;

const BUFFER_SIZE: usize = 1024 * 32;

#[tokio::main]
async fn main() {
    let atomic_counter = Arc::new(AtomicUsize::new(0));
    let mut stream = TcpStream::connect("127.0.0.1:7878").await.unwrap();
    // for _ in 0..12 {
    println!("new thread");
    {
        let atomic_counter = atomic_counter.clone();
        tokio::spawn(async move {
            loop {
                let mut buf = [0; BUFFER_SIZE];
                loop {
                    match stream.read(&mut buf[..]).await {
                        Ok(bytes) => {
                            atomic_counter.fetch_add(bytes, Ordering::Relaxed);
                        }
                        Err(e) => {
                            println!("{:?}", e);
                            tokio::time::sleep(Duration::from_millis(1000)).await;
                        }
                    }
                }
            }
        });
    }
    // }

    let mut now = std::time::Instant::now();
    loop {
        if now.elapsed() > Duration::from_secs(1) {
            println!(
                "received {} MB in {} ms",
                atomic_counter.load(Ordering::Relaxed) / 1024 / 1024,
                now.elapsed().as_millis()
            );
            now = std::time::Instant::now();
            atomic_counter.store(0, Ordering::Relaxed);
        }
    }
}
