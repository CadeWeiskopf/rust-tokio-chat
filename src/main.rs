use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_websockets::{Error, Message, ServerBuilder};
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Error> {
  let listener = TcpListener::bind("127.0.0.1:3000").await?;
  let clients: HashMap<Uuid, Arc<Mutex<SplitSink<tokio_websockets::WebSocketStream<TcpStream>, Message>>>> = HashMap::new();
  let clients_map = Arc::new(Mutex::new(clients));
  let messages_vec: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
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
      
      let (tx, rx) = ws_stream.split();

      let id = Uuid::new_v4();
      let client_rx_stream = Arc::new(Mutex::new(rx));
      let client_tx_stream = Arc::new(Mutex::new(tx));
      
      let mut clients_map_lock = clients_map_clone.lock().await;
      clients_map_lock.insert(id, client_tx_stream.clone());
      println!("\nNew connection to server {} \n Total Clients = {}\n", id, clients_map_lock.len());
      std::mem::drop(clients_map_lock);
      
      // spawn async task to listen for client messages
      // let clients_map_clone = clients_map.clone();
      let msg_vec_clone = messages_vec.clone();
      tokio::spawn(async move {
        let client_rx_stream_clone = client_rx_stream.clone();
        loop {
          let mut client_rx_lock = client_rx_stream_clone.lock().await;
          match client_rx_lock.next().await {
            Some(Ok(msg)) => {
              println!("{}: {:?}", id, msg);
              match msg.as_text() {
                Some(msg_str) => {
                  match parse_json_message(msg_str) {
                    Ok(msg_json) => {
                      eprintln!("Parsed message json: {}", msg_json);
                      let mut messages_vec_lock = msg_vec_clone.lock().await;
                      messages_vec_lock.push(msg_json);
                      println!("total messages = {}", messages_vec_lock.len());
                      std::mem::drop(messages_vec_lock);
                    },
                    Err(err) => {
                      eprintln!("Error parsing message as json: {}", err);
                    }
                  }   
                },
                None => {
                  eprintln!("None message as text");
                }
              }
              let client_tx_lock = client_tx_stream.lock().await;
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
      let clients_map_lock = clients_map_clone2.lock().await;
      for (id, client_stream) in clients_map_lock.iter() {
        println!("send {}", id);
        let mut c = client_stream.lock().await;
        let send_msg_result = c.send(Message::text("hello".to_string())).await;
        match send_msg_result {
          Ok(()) => {},
          Err(err) => {
            eprintln!("err sending message to client {}: {}", id, err);
          }
        }
        std::mem::drop(c);
      }
      std::mem::drop(clients_map_lock);
    }  
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