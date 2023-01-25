use std::collections::HashMap;
use std::net::UdpSocket;
use std::str::from_utf8;

struct DbEngine {
    store: HashMap<String, String>,
}

impl DbEngine {
    fn new() -> Self {
        DbEngine { store: HashMap::new() }
    }
    fn process_request(&mut self, req: &str) -> Option<String> {
        match req.split_once('=') {
            Some((key, val)) => { self.store.insert(key.to_owned(), val.to_owned()); }
            None => {
                if req == "version" {
                    return Some("version=1".to_string())
                }
                else if let Some(val) = self.store.get(req) {
                    return Some(format!("{req}={val}"))
                }
            }
        }
        None
    }
}

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:8000")?;
    let mut buf = [0;1000];
    let mut db = DbEngine::new();

    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        if let Ok(req) = from_utf8(&buf[..amt]) {
            if let Some(resp) = db.process_request(req) {
                socket.send_to(resp.as_bytes(), &src)?;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proces_basic_request() {
        let mut db = DbEngine::new();
        db.process_request("foo=bar");
        assert_eq!(db.process_request("foo"), Some("foo=bar".to_owned()));
    }
    #[test]
    fn proces_overwrite_request() {
        let mut db = DbEngine::new();
        db.process_request("foo=bar");
        db.process_request("foo=baz");
        assert_eq!(db.process_request("foo"), Some("foo=baz".to_owned()));
    }
    #[test]
    fn proces_special_request() {
        let mut db = DbEngine::new();
        db.process_request("version=2");
        assert_eq!(db.process_request("version"), Some("version=1".to_owned()));
    }
}
