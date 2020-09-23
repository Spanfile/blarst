use io::Write;
use rand::{
    distributions::Uniform,
    prelude::{Distribution, ThreadRng},
    thread_rng,
};
use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::{Duration, Instant},
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
    /// Try and blast at a certain rate of queries per second. A value of 0 means to blast as fast as possible
    #[structopt(short, long, default_value)]
    rate: u64,
    /// Blast out only this many queries. A value of 0 means sending out infinitely many queries
    #[structopt(short, long, default_value)]
    count: u64,
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
    let mut last_sent = 0u128;
    let mut last_read = 0u128;
    let mut last_update_time = Instant::now();

    let blaster_rate = 100;
    let update_rate = 10;

    let mut blaster_ticker = time::interval(Duration::from_nanos(1_000_000_000 / blaster_rate));
    let mut update_ticker = time::interval(Duration::from_nanos(1_000_000_000 / update_rate));

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
                let elapsed = last_update_time.elapsed().as_nanos();
                last_update_time = Instant::now();

                let last_interval_sent = sent - last_sent;
                last_sent = sent;
                let send_rate = last_interval_sent * (1_000_000_000 / elapsed);

                let last_interval_read = read - last_read;
                last_read = read;
                let read_rate = last_interval_read * (1_000_000_000 / elapsed);

                print!("\r{} [{}/s] {} [{}/s]", sent, send_rate, read, read_rate);
                io::stdout().flush().unwrap();
            }
        }
    }
}

fn dns_datagram(buf: &mut [u8], rng: &mut ThreadRng) {
    header(&mut buf[..12]);
    question(&mut buf[12..], rng);
}

fn header(buf: &mut [u8]) {
    let header: [u8; 12] = [0, 42, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0];
    buf[..12].copy_from_slice(&header[..12]);
}

fn question(buf: &mut [u8], rng: &mut ThreadRng) {
    buf[0] = 4;
    let chars = Uniform::from(97..123);
    for i in 0..4 {
        buf[1 + i] = chars.sample(rng);
    }

    buf[5..].copy_from_slice(&[3, 99, 111, 109, 0, 0, 1, 0, 1]);
}
