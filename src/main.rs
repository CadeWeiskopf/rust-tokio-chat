use futures_util::{stream::SplitSink, SinkExt, StreamExt, TryStreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_websockets::{Error, Message, ServerBuilder};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};

#[tokio::main]
async fn main() -> Result<(), Error> {
  let listener = TcpListener::bind("127.0.0.1:3000").await?;
  let mut clients: HashMap<Uuid, Arc<Mutex<SplitSink<tokio_websockets::WebSocketStream<TcpStream>, Message>>>> = HashMap::new();
  let mut locks: HashMap<Uuid, MutexGuard<'static, tokio_websockets::WebSocketStream<tokio::net::TcpStream>>> = HashMap::new();  
  let clients_map = Arc::new(Mutex::new(clients));

  let clients_map_clone = clients_map.clone();
  let clients_map_clone2 = clients_map.clone();

  tokio::spawn(async move {
    println!("Server listening... {:?}", listener); 
    while let Ok((stream, _)) = listener.accept().await { 
      // server setup client stream
      let ws_stream_result = ServerBuilder::new()
        .accept(stream)
        .await;
      let ws_stream = match ws_stream_result {
        Ok(stream) => {
          stream
        },
        Err(err) => {
          eprintln!("Failed to accept WebSocket connection: {}", err);
          break
        }
      };
      
      let (mut tx, mut rx) = ws_stream.split();

      let id = Uuid::new_v4();
      let client_rx_stream = Arc::new(Mutex::new(rx));
      let client_tx_stream = Arc::new(Mutex::new(tx));
      
      let mut clients_map_lock = clients_map_clone.lock().await;
      clients_map_lock.insert(id, client_tx_stream.clone());
      println!("\nNew connection to server {} \n Total Clients = {}\n", id, clients_map_lock.len());
      std::mem::drop(clients_map_lock);
      
      // spawn async task to listen for client messages
      // let clients_map_clone = clients_map.clone();
      tokio::spawn(async move {
        let client_rx_stream_clone = client_rx_stream.clone();
        loop {
          let mut client_rx_lock = client_rx_stream_clone.lock().await;
          match client_rx_lock.next().await {
            Some(Ok(msg)) => {
              println!("{}: {:?}", id, msg);
              // client_lock.send(Message::text("hello".to_string())).await;
              let mut client_tx_lock = client_tx_stream.lock().await;
              client_tx_lock.send(Message::text("hello".to_string())).await;
              std::mem::drop(client_tx_lock);
            },
            Some(Err(err)) => {
              eprintln!("Error receiving message: {}", err);
            },
            None => {
              println!("{} disconnected", id);
              break;
            }
          }
          std::mem::drop(client_rx_lock);
        }
      });
    }

    Ok::<_, Error>(())
  });
  
  tokio::spawn(async move {
    loop {
      tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
      let mut clients_map_lock = clients_map_clone2.lock().await;
      for (id, client_stream) in clients_map_lock.iter() {
        println!("send {}", id);
        let mut c = client_stream.lock().await;
        c.send(Message::text("hello".to_string())).await;
        std::mem::drop(c);
      }
      std::mem::drop(clients_map_lock);
    }  
    Ok::<_, Error>(())
  });

  loop {
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
  }
}

fn parse_json_message(data: &str) -> serde_json::Result<serde_json::Value> {
  println!("parsing {}", data);
  let value: serde_json::Value = serde_json::from_str(data)?;
  Ok(value)
}