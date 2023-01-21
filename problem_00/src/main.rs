use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::{Read, Write};

fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    loop {
        let bytes_read = match stream.read(&mut buffer) {
            Ok(n) => n,
            Err(_) => break,
        };
        if bytes_read == 0 {
            break;
        }
        match stream.write(&buffer[..bytes_read]) {
            Ok(_) => (),
            Err(_) => {
                println!("Error writing to stream. Closing connection.");
                break;
            }
        }
    }
    println!("Closing connection.");
    stream.shutdown(std::net::Shutdown::Both).unwrap();
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:8000").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_client(stream));
            }
            Err(_) => continue,
        }
    }
}
