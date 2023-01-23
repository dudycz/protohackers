use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;

const WELCOME_MESSAGE: &'static str = "Welcome to budgetchat! What shall I call you?\n";

#[derive(Clone, Debug)]
struct Message {
    session_id: u32,
    msg: String,
}

impl Message {
    pub fn user_joined(session_id: u32, name: String) -> Self {
        let msg = format!("* {name} has entered the room\n");
        Self { session_id, msg }
    }
    pub fn user_left(session_id: u32, name: String) -> Self {
        let msg = format!("* {name} has left the room\n");
        Self { session_id, msg }
    }
    pub fn from_user(session_id: u32, name: String, msg: String) -> Self {
        let msg = format!("[{name}] {msg}");
        Self { session_id, msg }
    }
}

async fn handle_chat_user(
    name: String,
    session_id: u32,
    socket: &mut BufReader<TcpStream>,
    channel: &mut Sender<Message>,
) -> Result<()> {

    let mut rx = channel.subscribe();
    socket
        .write_all("* List of participants: ....\n".as_bytes())
        .await?;

    let mut text = String::new();
    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    Ok(ev) => {
                        if ev.session_id != session_id {
                            socket.write_all(ev.msg.as_bytes()).await?;
                        }
                    },
                    _ => break,
                }
            }
            msg = socket.read_line(&mut text) => {
                match msg {
                    Ok(1..) => {
                        channel.send(Message::from_user(session_id, name.clone(), text.clone())).unwrap();
                    }
                    _ => break,
                }
            }
        }
    }

    Ok(())
}

async fn handle_new_connection(
    session_id: u32,
    socket: TcpStream,
    mut channel: Sender<Message>,
) -> Result<()> {
    let mut name = String::new();
    let mut socket = BufReader::new(socket);

    socket.write_all(WELCOME_MESSAGE.as_bytes()).await?;

    socket.read_line(&mut name).await?;
    name.pop();

    channel.send(Message::user_joined(session_id, name.clone()));
    let _res = handle_chat_user(name.clone(), session_id, &mut socket, &mut channel).await;
    channel.send(Message::user_left(session_id, name.clone()));
    socket.shutdown().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    let (tx, _) = broadcast::channel(32);
    let mut session_id = 0;
    loop {
        session_id += 1;
        let (socket, _) = listener.accept().await?;
        tokio::spawn(handle_new_connection(session_id, socket, tx.clone()));
    }
}
