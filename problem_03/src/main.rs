use anyhow::Result;
use std::collections::BTreeMap;
use std::ops::Bound::Included;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

struct PriceDb {
    price_list: BTreeMap<i32, i32>,
}

impl PriceDb {
    fn new() -> Self {
        PriceDb {
            price_list: BTreeMap::new(),
        }
    }

    fn insert(&mut self, timestamp: i32, price: i32) {
        self.price_list.insert(timestamp, price);
    }

    fn query(&self, min_time: i32, max_time: i32) -> i32 {
        if min_time > max_time {
            return 0;
        }

        let (sum, count) = self
            .price_list
            .range((Included(min_time), Included(max_time)))
            .map(|(_, v)| (*v as i64, 1 as i64))
            .fold((0, 0), |acc, x| (acc.0 + x.0, acc.1 + x.1));

        if count == 0 {
            return 0;
        }

        (sum / count) as i32
    }
}

async fn process(mut socket: TcpStream) -> Result<()> {
    let mut payload = [0; 9];
    let mut db = PriceDb::new();
    loop {
        match socket.read_exact(&mut payload).await {
            Ok(_) => match payload[0] {
                b'I' => {
                    let timestamp = i32::from_be_bytes(payload[1..=4].try_into()?);
                    let price = i32::from_be_bytes(payload[5..].try_into()?);
                    db.insert(timestamp, price);
                }
                b'Q' => {
                    let timestamp_min = i32::from_be_bytes(payload[1..=4].try_into()?);
                    let timestamp_max = i32::from_be_bytes(payload[5..].try_into()?);
                    let avg = db.query(timestamp_min, timestamp_max);
                    socket.write_all(&avg.to_be_bytes()).await?;
                }
                _ => {
                    break;
                }
            },
            Err(_) => {
                break;
            }
        }
    }
    socket.shutdown().await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(process(socket));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_success() {
        let mut db = PriceDb::new();
        db.insert(100, 10);
        db.insert(200, 20);
        db.insert(300, 30);
        assert_eq!(db.query(150, 250), 20);
        assert_eq!(db.query(200, 500), 25);
    }

    #[test]
    fn test_query_out_of_range() {
        let db = PriceDb::new();
        assert_eq!(db.query(100, 100), 0);
    }

    #[test]
    fn test_query_wrong_range() {
        let mut db = PriceDb::new();
        db.insert(100, 10);
        db.insert(200, 20);
        assert_eq!(db.query(200, 100), 0);
    }
}
