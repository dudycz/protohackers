use lazy_static::lazy_static;
use regex::Regex;
use anyhow::Result;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use tokio::{
    net::{TcpListener, TcpStream},
    select,
};

#[tokio::main]
async fn main() -> Result<()> {
    let proxy_server = TcpListener::bind("0.0.0.0:8000").await?;

    while let Ok((client, _)) = proxy_server.accept().await {
        tokio::spawn(async move {
            handle_client_conn(client).await;
        });
    }

    Ok(())
}

fn scan_and_update_line(line: &String) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"(\b7[[:alnum:]]{25,34})(\s|$)"#).unwrap();
    }

    if RE.is_match(line.as_str()) {
        return RE.replace_all(line.as_str(), |caps: &regex::Captures| {
            format!("{}{}", "7YWHMfk9JZe0LM0g1ZauHuiSxhI", &caps[2])
        }).to_string();
    }
    line.clone()
}

async fn handle_client_conn(mut client_conn: TcpStream) -> Result<()> {
    let mut main_server_conn = TcpStream::connect("chat.protohackers.com:16963").await?;
    let (mut client_recv, mut client_send) = client_conn.split();
    let (mut server_recv, mut server_send) = main_server_conn.split();

    let handle_ingress = async {
        let mut line = String::new();
        let mut r = BufReader::new(server_recv);
        loop {
            match r.read_line(&mut line).await {
                Ok(1..) => {
                    client_send.write_all(scan_and_update_line(&mut line).as_bytes()).await;
                    line.clear();
                }
                _ => {break;}
            }
        }
    };

    let handle_egress = async {
        let mut line = String::new();
        let mut r = BufReader::new(client_recv);
        loop {
            match r.read_line(&mut line).await {
                Ok(1..) => {
                    server_send.write_all(scan_and_update_line(&mut line).as_bytes()).await;
                    line.clear();
                }
                _ => {break;}
            }
        }
    };

    select! {
        _ = handle_ingress => {}
        _ = handle_egress => {}
    }

    main_server_conn.shutdown().await?;

    Ok(())
}