use futures::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};

const HELP_MSG: &str = include_str!("shared/help-01.txt");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = TcpListener::bind("127.0.0.1:42069").await?;
    loop {
        let (tcp, _) = server.accept().await?;
        tokio::spawn(handle_user(tcp));
    }
}

async fn handle_user(mut tcp: TcpStream) -> anyhow::Result<()> {
    let (reader, writer) = tcp.split();
    let mut stream = FramedRead::new(reader, LinesCodec::new());
    let mut sink = FramedWrite::new(writer, LinesCodec::new());
    sink.send(HELP_MSG).await?;
    while let Some(Ok(mut msg)) = stream.next().await {
        if msg.starts_with("/help") {
            sink.send(HELP_MSG).await?;
        } else if msg.starts_with("/quit") {
            break;
        } else {
            msg.push_str(" ❤️");
            sink.send(msg).await?;
        }
    }
    Ok(())
}
