#![allow(dead_code,unused_imports,unused_macros)]

// RANDOM NAME GENERATION //

use compact_str::CompactString;

mod adjectives;
mod animals;
mod english;
mod rust;

use adjectives::ADJECTIVES;
use animals::ANIMALS;
use english::ENGLISH;
use english::EMOJIS as ENG_EMOJIS;
use rust::RUST;
use rust::EMOJIS as RUST_EMOJIS;

pub fn choose<T: Copy>(array: &[T]) -> T {
    array[fastrand::usize(..array.len())]
}

pub fn random_name() -> String {
    let adjective = choose(&ADJECTIVES);
    let animal = choose(&ANIMALS);
    format!("{adjective}{animal}")
}

pub struct NameGenerator {
    adj_idx: usize,
    adj_offset: usize,
    an_idx: usize,
    an_offset_idx: usize,
    an_offsets: Vec<usize>,
}

impl NameGenerator {
    pub fn new() -> Self {
        let mut an_offsets: Vec<usize> = (0..ANIMALS.len()).collect();
        fastrand::shuffle(&mut an_offsets);
        Self {
            adj_idx: 0,
            adj_offset: fastrand::usize(..ADJECTIVES.len()),
            an_idx: 0,
            an_offset_idx: 0,
            an_offsets,
        }
    }
    pub fn next(&mut self) -> CompactString {
        // keep iterating until we get an adj +
        // animal combo that is 8-12 characters
        let (adj, animal) = loop {
            let adj =
                ADJECTIVES[(self.adj_idx + self.adj_offset) % ADJECTIVES.len()];
            let animal = ANIMALS[(self.an_idx
                + self.an_offsets[self.an_offset_idx])
                % ANIMALS.len()];

            self.adj_idx += 1;
            self.adj_idx %= ADJECTIVES.len();
            self.an_idx += 1;
            self.an_idx %= ANIMALS.len();
            if self.adj_idx == 0 {
                self.an_idx = 0;
                self.an_offset_idx += 1;
                self.an_offset_idx %= self.an_offsets.len();
            }

            if (8..=12).contains(&(adj.len() + animal.len())) {
                break (adj, animal);
            }
        };

        let mut name = CompactString::new(adj);
        name.push_str(animal);
        name
    }
}

// MACROS //

#[macro_export]
macro_rules! b {
    ($result:expr) => {
        match $result {
            Ok(ok) => ok,
            Err(err) => break Err(err.into()),
        }
    }
}

// COMMAND LINE //

use std::net::{IpAddr, SocketAddr, Ipv4Addr};
use clap::Parser;

pub const DEFAULT_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
pub const DEFAULT_PORT: u16 = 42069;

#[derive(Parser)]
#[command(long_about = None)]
struct Cli {
    #[arg(short, long, default_value_t = DEFAULT_IP)]
    ip: IpAddr,

    #[arg(short, long, default_value_t = DEFAULT_PORT)]
    port: u16,
}

pub fn parse_socket_addr() -> SocketAddr {
    let cli = Cli::parse();
    SocketAddr::new(cli.ip, cli.port)
}

// LOGGING //

const LOGS_DIR: &str = "logs";

use std::io;
use tracing_appender::{non_blocking::WorkerGuard, rolling::{RollingFileAppender, Rotation}};
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt};

pub fn stdout_logging() {
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::Layer::new().without_time().compact().with_ansi(true).with_writer(io::stdout));
    tracing::subscriber::set_global_default(subscriber)
            .expect("Unable to set a global subscriber");
}

pub fn file_logging(rotation: Rotation, log_file: &str) -> WorkerGuard {
    let _ = std::fs::create_dir(LOGS_DIR);
    let file_appender = RollingFileAppender::new(rotation, LOGS_DIR, log_file);
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::Layer::new().compact().with_ansi(false).with_writer(non_blocking));
    tracing::subscriber::set_global_default(subscriber)
            .expect("Unable to set a global subscriber");
    guard
}

// RANDOM MSG GENERATION //

pub fn random_english_msg() -> String {
    let mut msg = String::with_capacity(32);
    let words = fastrand::u8(2..=10);
    for _ in 1..words {
        msg.push_str(choose(&ENGLISH));
        msg.push(' ');
    }
    msg.push_str(choose(&ENGLISH));
    if fastrand::bool() {
        msg.push(' ');
        msg.push(choose(&ENG_EMOJIS));
    }
    msg
}

pub fn random_rust_msg() -> String {
    let mut msg = String::with_capacity(32);
    let words = fastrand::u8(2..=8);
    for _ in 1..words {
        msg.push_str(choose(&RUST));
        msg.push(' ');
    }
    msg.push_str(choose(&RUST));
    if fastrand::bool() {
        msg.push(' ');
        msg.push_str(choose(&RUST_EMOJIS));
    }
    msg
}

// MISC //

pub fn valid_name(name: Option<&str>) -> bool {
    match name {
        None => false,
        Some(name) => {
            if name.len() < 2 {
                return false;
            }
            if name.len() > 20 {
                return false;
            }
            name
                .chars()
                .all(|c| char::is_ascii_alphanumeric(&c) || c == '-' || c == '_')
        }
    }
}

pub fn connection_refused(tried: SocketAddr) -> String {
    let mut msg = format!("No server listening on {tried}\n");
    msg.push_str("Try running: cargo run --release --bin chat-server");
    if tried.port() != DEFAULT_PORT || tried.ip() != DEFAULT_IP {
        msg.push_str(" --");
        if tried.ip() != DEFAULT_IP {
            msg.push_str(" -i ");
            msg.push_str(&tried.ip().to_string());
        }
        if tried.port() != DEFAULT_PORT {
            msg.push_str(" -p ");
            msg.push_str(&tried.port().to_string());
        }
    }
    msg
}
