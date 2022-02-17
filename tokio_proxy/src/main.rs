#![feature(slice_as_chunks)]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;
use std::vec;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::net::UdpSocket;
use tokio::sync::broadcast::{self, Receiver};
use tokio::sync::RwLock;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// whether to request lower requirements for the GPU
    #[structopt(short, long = "address", default_value = "127.0.0.1:34254")]
    address: String,
}

const BUFFER_SIZE: usize = 1024 * 32;

async fn upd_stuff(
    udp_socket: Arc<UdpSocket>,
    sender: broadcast::Sender<Vec<u8>>,
    perf_counter: Arc<AtomicUsize>,
) {
    let mut buf = [0; BUFFER_SIZE];
    let mut total_buffer = [0; BUFFER_SIZE];
    loop {
        match udp_socket.recv_from(&mut buf).await {
            Ok((amt, _src)) => {
                let buf = &mut buf[..amt];
                // let new_index = if amt + current_buffer_index > BUFFER_SIZE {
                //     BUFFER_SIZE
                // } else {
                //     amt + current_buffer_index
                // };
                // total_buffer[current_buffer_index..new_index]
                //     .copy_from_slice(&buf[..(new_index - current_buffer_index)]);

                // current_buffer_index = new_index;
                // if current_buffer_index == BUFFER_SIZE {
                // let mut vecu32 = Vec::with_capacity(BUFFER_SIZE / 4);
                // for chunk in total_buffer.as_chunks().0 {
                //     let u32: u32 = u32::from_le_bytes(*chunk);
                //     vecu32.push(u32);
                // }
                // sender.lock().unwrap().broadcast(Vec::from(buf));
                sender.send(Vec::from(buf)).unwrap();
                // tokio::time::sleep(Duration::from_millis(1)).await;

                perf_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                // }

                // sender.send(()).unwrap();
                // println!("Received {} bytes from {}", amt, src);
                // println!("{}", String::from_utf8_lossy(buf));
            }
            Err(_) => todo!(),
        };
    }
}

#[tokio::main]
async fn main() {
    let opt: Opt = Opt::from_args();

    let (tx, mut rx1) = broadcast::channel(16);

    let tcp_listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();
    // let tcp_pool = ThreadPool::new(4, bus.clone());

    let udp_socket = Arc::new(UdpSocket::bind(opt.address).await.unwrap());

    let atomic_counter = Arc::new(AtomicUsize::new(0));
    for _ in 0..12 {
        println!("new thread");
        let s = udp_socket.clone();
        // let bus = bus.clone();
        let counter = atomic_counter.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            upd_stuff(s, tx, counter).await;
        });
    }

    // rx.recv().expect("Could not receive from channel.");
    tokio::spawn(async move {
        loop {
            match tcp_listener.accept().await {
                Ok((socket, addr)) => {
                    println!("new client: {:?}", addr);
                    let mut rx2 = tx.subscribe();
                    tokio::spawn(async move {
                        println!("spawning thread");
                        handle_connection(socket, rx2).await;
                        // handle_connection(socket).await;
                    });
                }
                Err(e) => println!("couldn't get client: {:?}", e),
            }

            // tcp_pool.execute(move |rx: &mut BusReader<Vec<u8>>| {
            //     handle_connection(stream, rx);
            // });
        }
    });

    let mut now = std::time::Instant::now();
    loop {
        if now.elapsed() > Duration::from_secs(1) {
            println!(
                "received {} packets, aka {} MB in {} ms",
                atomic_counter.load(Ordering::Relaxed),
                atomic_counter.load(Ordering::Relaxed) * BUFFER_SIZE / 1024 / 1024,
                now.elapsed().as_millis()
            );
            now = std::time::Instant::now();
            atomic_counter.store(0, Ordering::Relaxed);
        }
    }
}

async fn handle_connection(mut socket: TcpStream, mut rx: Receiver<Vec<u8>>) {
    println!("new connection");
    'outer: loop {
        let vec = loop {
            match rx.recv().await {
                Ok(v) => break v,
                Err(e) => {
                    println!("vec {:?}", e);
                    break 'outer;
                }
            }
        };

        // let response = format!("{:?}\n", vec);

        // let vec = &vec.read().await;

        // println!("hi");

        // match socket.write_all(response.as_bytes()).await {
        // println!("send");
        // let vec = vec![0; BUFFER_SIZE];
        match socket.write_all(&vec[..]).await {
            Ok(_) => {}
            Err(e) => {
                println!("Failed to write: {:?}", e);
                break;
            }
        };
        // println!("sent");
    }
    println!("Client disconnected.");
}
