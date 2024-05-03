#![allow(unused)]

use std::cmp::max;
use std::iter::repeat_with;
use std::net::SocketAddr;
use std::ops::{AddAssign, RangeInclusive};
use std::time::{Duration, Instant};
use futures::SinkExt;
use chat_server::{choose, connection_refused, parse_socket_addr, random_english_msg, random_rust_msg, stdout_logging};
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpStream,
};
use tokio::task::JoinSet;
use tokio_stream::StreamExt;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

struct Bot<M> {
    msgs: M,
    msg_delay: RangeInclusive<u64>,
    sink: FramedWrite<OwnedWriteHalf, LinesCodec>,
    stream: FramedRead<OwnedReadHalf, LinesCodec>,
    stats: Stats,
}

#[derive(Default, Debug)]
struct Stats {
    sent_bytes: usize,
    got_bytes: usize,
    sent_msgs: usize,
    got_msgs: usize,
}

impl AddAssign for Stats {
    fn add_assign(&mut self, rhs: Self) {
        self.sent_bytes += rhs.sent_bytes;
        self.sent_msgs += rhs.sent_msgs;
        self.got_bytes += rhs.got_bytes;
        self.got_msgs += rhs.got_msgs;
    }
}

impl<M: Iterator<Item = String>> Bot<M> {
    async fn new(addr: SocketAddr, msgs: M, msg_delay: RangeInclusive<u64>) -> anyhow::Result<Self> {
        let conn = TcpStream::connect(addr).await?;
        let (reader, writer) = conn.into_split();
        let sink = FramedWrite::new(writer, LinesCodec::new());
        let stream = FramedRead::new(reader, LinesCodec::new());
        Ok(Self {
            msgs,
            msg_delay,
            sink,
            stream,
            stats: Stats::default(),
        })
    }
    async fn chat(mut self) -> anyhow::Result<Stats> {
        for msg in self.msgs {
            let msg_len = msg.len();
            self.sink.send(msg).await?;
            self.stats.sent_bytes += msg_len + 1;
            self.stats.sent_msgs += 1;
            let sleep = tokio::time::sleep(
                Duration::from_millis(fastrand::u64(self.msg_delay.clone()))
            );
            tokio::pin!(sleep);
            loop {
                tokio::select! {
                    option = self.stream.next() => {
                        if let Some(result) = option {
                            let msg = result?;
                            self.stats.got_bytes += msg.len() + 1;
                            self.stats.got_msgs += 1;
                        }
                    },
                    _ = &mut sleep => {
                        break;
                    },
                };
            }
        }
        Ok(self.stats)
    }
}

struct Simple {
    send_msgs: usize,
    msgs_sent: usize,
}

impl Simple {
    fn new() -> Self {
        Self {
            send_msgs: 100,
            msgs_sent: 0,
        }
    }
}

impl Iterator for Simple {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        if self.msgs_sent >= self.send_msgs {
            return None;
        }
        let msg = if self.msgs_sent == self.send_msgs - 1 {
            "/quit".to_owned()
        } else {
            random_english_msg()
        };
        self.msgs_sent += 1;
        Some(msg)
    }
}

struct Rusty {
    send_msgs: usize,
    msgs_sent: usize,
}

impl Rusty {
    fn new() -> Self {
        Self {
            send_msgs: 100,
            msgs_sent: 0,
        }
    }
}

impl Iterator for Rusty {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        if self.msgs_sent >= self.send_msgs {
            return None;
        }
        let msg = if self.msgs_sent == 0 {
            "/join rust".to_owned()
        } else if self.msgs_sent == self.send_msgs - 1 {
            "/quit".to_owned()
        } else {
            random_rust_msg()
        };
        self.msgs_sent += 1;
        Some(msg)
    }
}

struct StressTest {
    send_msgs: usize,
    msgs_sent: usize,
}

impl StressTest {
    fn new() -> Self {
        Self {
            send_msgs: 100000,
            msgs_sent: 0,
        }
    }
}

impl Iterator for StressTest {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        if self.msgs_sent >= self.send_msgs {
            return None;
        }
        let msg = if self.msgs_sent == 0 {
            "/join stress-test".to_owned()
        } else if self.msgs_sent == self.send_msgs - 1 {
            "/quit".to_owned()
        } else {
            random_english_msg()
        };
        self.msgs_sent += 1;
        Some(msg)
    }
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let addr = parse_socket_addr();
    stdout_logging();
    let conn = match TcpStream::connect(addr).await {
        Ok(conn) => conn,
        Err(err) => {
            match err.kind() {
                std::io::ErrorKind::ConnectionRefused => {
                    tracing::error!("{}", connection_refused(addr));
                    std::process::exit(1)
                }
                // got unexpected err, re-throw
                _ => Err(err)?,
            }
        }
    };
    drop(conn);

    let mut stats = Stats::default();
    let mut set = JoinSet::new();
    tracing::info!("spawning bots");

    // spawn 3 simple bots
    for _ in 0..3 {
        let bot = Bot::new(addr, Simple::new(), 2000..=4000).await?;
        set.spawn(bot.chat());
    }

    // spawn 3 rusty bots
    for _ in 0..3 {
        let bot = Bot::new(addr, Rusty::new(), 2000..=4000).await?;
        set.spawn(bot.chat());
    }

    // spawn 100 stress-test bots
    for _ in 0..100 {
        let bot = Bot::new(addr, StressTest::new(), 100..=200).await?;
        set.spawn(bot.chat());
    }

    tracing::info!("waiting for all bots to join");
    while let Some(join_result) = set.join_next().await {
        let chat_result = join_result?;
        match chat_result {
            Ok(bot_stats) => stats += bot_stats,
            Err(err) => {
                tracing::error!("{err}");
            }
        }
    }

    tracing::info!("sent bytes - {}", stats.sent_bytes);
    tracing::info!("got bytes  - {}", stats.got_bytes);
    tracing::info!("sent msgs  - {}", stats.sent_msgs);
    tracing::info!("got msgs   - {}", stats.sent_msgs);

    Ok(())
}
