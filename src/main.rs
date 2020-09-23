use circular_queue::CircularQueue;
use io::Write;
use rand::{
    distributions::Uniform,
    prelude::{Distribution, ThreadRng},
    thread_rng,
};
use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};
use structopt::StructOpt;
use tokio::{net::UdpSocket, time};

const APP_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Debug, StructOpt)]
#[structopt(name = APP_NAME, about, author)]
struct Opts {
    /// The blasting target's address
    #[structopt(parse(try_from_str))]
    target: IpAddr,
    /// Target a certain port
    #[structopt(short, long, default_value = "53")]
    port: u16,
    /// Try and blast at a certain rate of queries per second. A value of 0 means to blast as fast as possible, which
    /// in reality means trying to send a query once every nanosecond.
    #[structopt(short, long, default_value)]
    rate: u64,
    /// Blast out only this many queries. A value of 0 means sending out infinitely many queries
    #[structopt(short, long, default_value)]
    count: u64,
    /// How many times a second to update the status display.
    #[structopt(short, long, default_value = "10")]
    update: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = Opts::from_args();
    println!("{:?}", opts);

    let socket = UdpSocket::bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)).await?;
    socket.connect((opts.target, opts.port)).await?;
    let (mut reader, mut blaster) = socket.split();
    let mut buf = [0u8; 128];

    let mut sent = 0u128;
    let mut read = 0u128;
    let mut send_queue = CircularQueue::with_capacity(opts.update as usize);

    let blaster_rate = if opts.rate == 0 { 1_000_000_000 } else { opts.rate };
    let mut blaster_ticker = time::interval(Duration::from_nanos(1_000_000_000 / blaster_rate));
    let mut update_ticker = time::interval(Duration::from_nanos(1_000_000_000 / opts.update));

    loop {
        tokio::select! {
            _ = blaster_ticker.tick() => {
                let mut datagram = [0u8; 26];
                let mut rng = thread_rng();

                dns_datagram(&mut datagram, &mut rng);
                blaster.send(&datagram).await.unwrap();
                sent += 1;
            }
            _ = reader.recv(&mut buf) => {
                read += 1;
            }
            _ = update_ticker.tick() => {
                let send_rate = if let Some(oldest) = send_queue.push(sent) {
                    sent - oldest
                } else {
                    sent - send_queue.asc_iter().next().unwrap()
                };
                let read_rate = 0;

                let send_read_percentage = if sent == 0 { 1.0 } else { read as f64 / sent as f64 } * 100.0;
                print!("\rr/s: {: >9}/{: >9} [{: >6.2}%] s:[{: >7}/s] r:[{: >7}/s]", read, sent, send_read_percentage, send_rate, read_rate);
                io::stdout().flush().unwrap();
            }
        }
    }
}

fn dns_datagram(buf: &mut [u8], rng: &mut ThreadRng) {
    header(&mut buf[..12], rng);
    question(&mut buf[12..], rng);
}

fn header(buf: &mut [u8], rng: &mut ThreadRng) {
    let id = Uniform::new(1u16, u16::MAX).sample(rng);
    buf[0] = (id >> 8) as u8;
    buf[1] = (id & 0xff) as u8;
    buf[2..].copy_from_slice(&[1, 0, 0, 1, 0, 0, 0, 0, 0, 0]);
}

fn question(buf: &mut [u8], rng: &mut ThreadRng) {
    // generate a question for <four random lowercase ascii characters>.com
    buf[0] = 4;
    let chars = Uniform::from(97..123);
    for i in 0..4 {
        buf[1 + i] = chars.sample(rng);
    }

    buf[5..].copy_from_slice(&[3, 99, 111, 109, 0, 0, 1, 0, 1]);
}
