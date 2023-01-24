use anyhow::Result;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

type UserList = Arc<Mutex<HashMap<u32, String>>>;


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
        let msg = format!("[{name}] {msg}\n");
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
    let (rd, mut wr) = io::split(socket);
    let rd = BufReader::new(rd);
    let mut line_reader = rd.lines();

    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    Ok(ev) => {
                        if ev.session_id != session_id {
                            wr.write_all(ev.msg.as_bytes()).await?;
                        }
                    },
                    _ => break,
                }
            },
            result = line_reader.next_line() => {
                match result {
                    Ok(Some(text))  => {
                        let text = text.trim();
                        channel.send(Message::from_user(session_id, name.clone(), text.to_owned())).unwrap();
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
    users: UserList
) -> Result<()> {
    let mut name = String::new();
    let mut socket = BufReader::new(socket);

    socket.write_all(WELCOME_MESSAGE.as_bytes()).await?;

    socket.read_line(&mut name).await?;
    let name = name.trim().to_owned();

    if name.chars().all(|c| c.is_ascii_alphanumeric()) && !name.is_empty() {
        let names = users.lock().await.values().cloned().collect::<Vec<String>>();
        users.lock().await.insert(session_id, name.clone());
        let msg = format!("* List of participants: {:?}\n", names);
        socket.write_all(msg.as_bytes()).await?;

        channel.send(Message::user_joined(session_id, name.clone())).ok();
        let _res = handle_chat_user(name.clone(), session_id, &mut socket, &mut channel).await;
        users.lock().await.remove(&session_id);
        channel.send(Message::user_left(session_id, name.clone())).ok();
    }
    socket.shutdown().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    let (tx, _) = broadcast::channel(50);
    let users = Arc::new(Mutex::new(HashMap::new()));
    let mut session_id = 0;
    loop {
        session_id += 1;
        let (socket, _) = listener.accept().await?;
        tokio::spawn(handle_new_connection(session_id, socket, tx.clone(), users.clone()));
    }
}
