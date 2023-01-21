use tokio::io::{self, AsyncWriteExt, BufReader};
use tokio::io::AsyncBufReadExt;
use tokio::net::{TcpListener, TcpStream};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize, Serialize)]
struct Request {
    method: String,
    number: f64,
}

fn is_prime(n: f64) -> bool {
    if n <= 1.0 || n.fract() != 0.0 {
        return false;
    }
    let limit = (n.sqrt() + 1.0) as i64;
    let n = n as i64;
    for a in 2..limit {
        if n % a == 0 {
            return false; // if it is not the last statement you need to use `return`
        }
    }
    true // last value to return
}

fn execute_request(request: &String) -> Result<String, String> {
    if let Ok(req) = serde_json::from_str::<Request>(&request) {
        if req.method == "isPrime" {
            let result = json!({
                "method": "isPrime",
                "prime": is_prime(req.number)
            });
            return Ok(result.to_string() + "\n")
        }
    }
    Err("Malformed response\n".to_string())
}

async fn process(socket: TcpStream) {
    let (rd, mut wr) = io::split(socket);
    //let mut buf_reader = BufReader::new(rd);
    let mut lines = BufReader::new(rd).lines();

    loop {
        let line= lines.next_line().await;
        match line {
            // No more lines in this stream
            Ok(None) => {
                println!("[ERR] Remote peer closed socket");
                return
            },
            Ok(Some(line)) => {
                match execute_request(&line) {
                    Ok(reply) => {
                        if wr.write_all(reply.as_bytes()).await.is_err() {
                            println!("[ERR] Unexpected write error");
                            return;
                        }
                    },
                    Err(reply) => {
                        println!("[ERR] Malformed request: {:?}", line);
                        let _res = wr.write_all(reply.as_bytes()).await;
                        return
                    }
                }
            }
            Err(_) => {
                // IO error while reading
                println!("[ERR] Unexpected socket error");
                return;
            }
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    println!("Server bound to 0.0.0.0:8000");

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection established from {:?}", addr);

        tokio::spawn(async move {
            process(socket).await;
        });
    }
}