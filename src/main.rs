use futures_util::{SinkExt, StreamExt, TryStreamExt};
use tokio::net::TcpListener;
use tokio_websockets::{Error, Message, ServerBuilder};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Error> {
  let listener = TcpListener::bind("127.0.0.1:3000").await?;
  let mut clients: HashMap<Uuid, Arc<Mutex<tokio_websockets::WebSocketStream<tokio::net::TcpStream>>>> = HashMap::new();

  tokio::spawn(async move {
    /*
     * Spawn server start listening
     */
    println!("Server listening... {:?}", listener); 
    while let Ok((stream, _)) = listener.accept().await { 
      /*
       * server setup client stream
       */
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
      let id = Uuid::new_v4();
      let client_stream = Arc::new(Mutex::new(ws_stream));
      clients.insert(id, client_stream.clone());
      println!("\nNew connection to server {} \n Total Clients = {}\n", id, clients.len());
      
      // spawn async task to listen for client messages
      tokio::spawn(async move {
        let mut client_lock = client_stream.lock().await;
        loop {
          match client_lock.next().await {
            Some(Ok(msg)) => {
              println!("{}: {:?}", id, msg);
            },
            Some(Err(err)) => {
              eprintln!("Error receiving message: {}", err);
            },
            None => {
              println!("{} disconnected", id);
              break;
            }
          }
        }
      });
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