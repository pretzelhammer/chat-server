use std::{collections::HashSet, sync::Arc};
use compact_str::{CompactString, ToCompactString};
use dashmap::{DashMap, DashSet};
use futures::{SinkExt, StreamExt};
use tokio::{net::{TcpListener, TcpStream}, sync::broadcast::{self, Sender}};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[path ="shared/lib.rs"]
mod lib;
use lib::{b, NameGenerator};

const MAIN: &str = "main";
const HELP_MSG: &str = include_str!("shared/help-05.txt");

#[derive(Clone)]
#[repr(transparent)]
struct Names(Arc<DashSet<CompactString>>);

impl Names {
    fn new() -> Self {
        Self(Arc::new(DashSet::with_capacity(32)))
    }
    fn insert(&self, name: CompactString) -> bool {
        self.0.insert(name)
    }
    fn remove(&self, name: &str) -> bool {
        self.0.remove(name).is_some()
    }
    fn get_unique(&self, name_generator: &mut NameGenerator) -> CompactString {
        let mut name = name_generator.next();
        while !self.0.insert(name.clone()) {
            name = name_generator.next();
        }
        name
    }
}

#[derive(Clone, Debug)]
enum RoomMsg {
    Joined(CompactString),
    Left(CompactString),
    Msg(Arc<str>),
}

struct Room {
    tx: Sender<RoomMsg>,
    users: HashSet<CompactString>,
}

impl Room {
    fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        let users = HashSet::with_capacity(8);
        Self {
            tx,
            users,
        }
    }
}

#[derive(Clone)]
#[repr(transparent)]
struct Rooms(Arc<DashMap<CompactString, Room>>);

impl Rooms {
    fn new() -> Self {
        Self(Arc::new(DashMap::with_capacity(8)))
    }
    fn join(&self, room_name: &str, user_name: &str) -> Sender<RoomMsg> {
        let mut room = self.0.entry(room_name.into()).or_insert(Room::new());
        room.users.insert(user_name.into());
        room.tx.clone()
    }
    fn leave(&self, room_name: &str, user_name: &str) {
        let mut delete_room = false;
        if let Some(mut room) = self.0.get_mut(room_name) {
            room.users.remove(user_name);
            delete_room = room.tx.receiver_count() <= 1;
        }
        if delete_room {
            self.0.remove(room_name);
        }
    }
    fn change(&self, prev_room: &str, next_room: &str, user_name: &str) -> Sender<RoomMsg> {
        self.leave(prev_room, user_name);
        self.join(next_room, user_name)
    }
    fn change_name(&self, room_name: &str, prev_name: &str, new_name: &str) {
        if let Some(mut room) = self.0.get_mut(room_name) {
            room.users.remove(prev_name);
            room.users.insert(new_name.to_compact_string());
        }
    }
    fn list(&self) -> Vec<(CompactString, usize)> {
        let mut list: Vec<_> = self
            .0
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().tx.receiver_count()))
            .collect();
        list.sort_by(|a, b| {
            use std::cmp::Ordering;
            match b.1.cmp(&a.1) {
                Ordering::Equal => a.0.cmp(&b.0),
                ordering => ordering,
            }
        });
        list
    }
    fn list_users(&self, room_name: &str) -> Option<Vec<CompactString>> {
        self
            .0
            .get(room_name)
            .map(|room| {
                let mut users = room
                    .users
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>();
                users.sort();
                users
            })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = TcpListener::bind("127.0.0.1:42069").await?;
    let mut name_generator = NameGenerator::new();
    let names = Names::new();
    let rooms = Rooms::new();
    loop {
        let (tcp, _) = server.accept().await?;
        let unique_name = names.get_unique(&mut name_generator);
        tokio::spawn(handle_user(tcp, names.clone(), rooms.clone(), unique_name));
    }
}

async fn handle_user(
    mut tcp: TcpStream,
    names: Names,
    rooms: Rooms,
    mut name: CompactString,
) -> anyhow::Result<()> {
    let (reader, writer) = tcp.split();
    let mut stream = FramedRead::new(reader, LinesCodec::new());
    let mut sink = FramedWrite::new(writer, LinesCodec::new());
    sink.send(format!("{HELP_MSG}\nYou are {name}")).await?;
    let mut room_name = CompactString::from(MAIN);
    let mut room_tx = rooms.join(&room_name, &name);
    let mut room_rx = room_tx.subscribe();
    let _ = room_tx.send(RoomMsg::Joined(name.clone()));
    let result: anyhow::Result<()> = loop {
        tokio::select! {
            user_msg = stream.next() => {
                let user_msg = match user_msg {
                    Some(msg) => b!(msg),
                    None => break Ok(()),
                };
                if user_msg.starts_with("/help") {
                    b!(sink.send(HELP_MSG).await);
                } else if user_msg.starts_with("/name") {
                    let new_name = user_msg
                        .split_ascii_whitespace()
                        .nth(1)
                        .unwrap()
                        .to_compact_string();
                    let changed_name = names.insert(new_name.clone());
                    if changed_name {
                        rooms.change_name(&room_name, &name, &new_name);
                        let msg = format!("{name} is now {new_name}");
                        let msg: Arc<str> = Arc::from(msg.as_str());
                        b!(room_tx.send(RoomMsg::Msg(msg)));
                        name = new_name;
                    } else {
                        b!(sink.send(format!("{new_name} is already taken")).await);
                    }
                } else if user_msg.starts_with("/join") {
                    let new_room = user_msg
                        .split_ascii_whitespace()
                        .nth(1)
                        .unwrap()
                        .to_compact_string();
                    if new_room == room_name {
                        b!(sink.send(format!("You are in {room_name}")).await);
                        continue;
                    }
                    b!(room_tx.send(RoomMsg::Left(name.clone())));
                    room_tx = rooms.change(&room_name, &new_room, &name);
                    room_rx = room_tx.subscribe();
                    room_name = new_room;
                    b!(room_tx.send(RoomMsg::Joined(name.clone())));
                } else if user_msg.starts_with("/rooms") {
                    let rooms_list = rooms.list();
                    let mut rooms_msg = String::with_capacity(rooms_list.len() * 15);
                    rooms_msg.push_str("Rooms - ");
                    for room in rooms_list {
                        rooms_msg.push_str(&room.0);
                        rooms_msg.push_str(" (");
                        rooms_msg.push_str(&room.1.to_string());
                        rooms_msg.push_str("), ");
                    }
                    // pop off trailing comma + space
                    rooms_msg.pop();
                    rooms_msg.pop();
                    b!(sink.send(rooms_msg).await);
                } else if user_msg.starts_with("/users") {
                    let users_list = rooms.list_users(&room_name).unwrap();
                    let mut users_msg = String::with_capacity(users_list.len() * 15);
                    users_msg.push_str("Users - ");
                    for user in users_list {
                        users_msg.push_str(&user);
                        users_msg.push_str(", ");
                    }
                    // pop off trailing comma + space
                    users_msg.pop();
                    users_msg.pop();
                    b!(sink.send(users_msg).await);
                } else if user_msg.starts_with("/quit") {
                    break Ok(());
                } else {
                    let msg = format!("{name}: {user_msg}");
                    let msg: Arc<str> = Arc::from(msg.as_str());
                    b!(room_tx.send(RoomMsg::Msg(msg)));
                }
            },
            peer_msg = room_rx.recv() => {
                let peer_msg = b!(peer_msg);
                match peer_msg {
                    RoomMsg::Joined(peer_name) => {
                        let msg = if name == peer_name {
                            format!("You joined {room_name}")
                        } else {
                            format!("{peer_name} joined")
                        };
                        b!(sink.send(msg).await);
                    },
                    RoomMsg::Left(peer_name) => {
                        let msg = if name == peer_name {
                            format!("You left {room_name}")
                        } else {
                            format!("{peer_name} left")
                        };
                        b!(sink.send(msg).await);
                    },
                    RoomMsg::Msg(msg) => {
                        b!(sink.send(msg).await);
                    },
                };
            },
        }
    };
    let _ = room_tx.send(RoomMsg::Left(name.clone()));
    rooms.leave(&room_name, &name);
    names.remove(&name);
    result
}
