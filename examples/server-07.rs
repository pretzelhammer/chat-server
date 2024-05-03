use futures::{SinkExt, StreamExt};
use tokio::{net::{TcpListener, TcpStream}, sync::broadcast::{self, Sender}};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

const HELP_MSG: &str = include_str!("help-01.txt");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = TcpListener::bind("127.0.0.1:42069").await?;
    let (tx, _) = broadcast::channel::<String>(32);
    loop {
        let (tcp, _) = server.accept().await?;
        tokio::spawn(handle_user(tcp, tx.clone()));
    }
}

async fn handle_user(mut tcp: TcpStream, tx: Sender<String>) -> anyhow::Result<()> {
    let (reader, writer) = tcp.split();
    let mut stream = FramedRead::new(reader, LinesCodec::new());
    let mut sink = FramedWrite::new(writer, LinesCodec::new());
    let mut rx = tx.subscribe();
    sink.send(HELP_MSG).await?;
    while let Some(Ok(mut user_msg)) = stream.next().await {
        if user_msg.starts_with("/help") {
            sink.send(HELP_MSG).await?;
        } else if user_msg.starts_with("/quit") {
            break;
        } else {
            user_msg.push_str(" ❤️");
            tx.send(user_msg)?;
        }
        let peer_msg = rx.recv().await?;
        sink.send(peer_msg).await?;
    }
    Ok(())
}
