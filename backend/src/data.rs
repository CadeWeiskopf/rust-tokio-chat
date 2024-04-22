use std::sync::Arc;
use tokio::sync::Mutex;
use futures_util::{stream::SplitSink, stream::SplitStream};
use tokio::net::TcpStream;
use tokio_websockets::{Message, WebSocketStream};

pub struct User {
  pub tx_stream: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>,
  pub rx_stream: Arc<Mutex<SplitStream<WebSocketStream<TcpStream>>>>,
  pub name: String,
  pub name_key: String
}

pub fn parse_json_message(data: &str) -> serde_json::Result<serde_json::Value> {
  println!("parsing {}", data);
  let value: serde_json::Value = serde_json::from_str(data)?;
  Ok(value)
}
