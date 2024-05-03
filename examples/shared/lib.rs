#![allow(dead_code,unused_imports,unused_macros)]

// RANDOM NAME GENERATION //

mod adjectives;
mod animals;

use compact_str::CompactString;
use adjectives::ADJECTIVES;
use animals::ANIMALS;

pub fn random_name() -> String {
    let adjective = fastrand::choice(ADJECTIVES).unwrap();
    let animal = fastrand::choice(ANIMALS).unwrap();
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

macro_rules! b {
    ($result:expr) => {
        match $result {
            Ok(ok) => ok,
            Err(err) => break Err(err.into()),
        }
    }
}
pub(crate) use b;


// COMMAND LINE ARGS //

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

use std::io;
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt};

pub fn setup_logging() {
    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::Layer::new().without_time().compact().with_ansi(true).with_writer(io::stdout));
    tracing::subscriber::set_global_default(subscriber)
            .expect("Unable to set a global subscriber");
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
