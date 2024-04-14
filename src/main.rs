use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_websockets::{Error, Message, ServerBuilder};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Error> {
  let listener = TcpListener::bind("127.0.0.1:3000").await?;

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
      let mut ws_stream = match ws_stream_result {
        Ok(stream) => {
          stream
        },
        Err(err) => {
          eprintln!("Failed to accept WebSocket connection: {}", err);
          break
        }
      };
      let id = Uuid::new_v4();
      println!("New connection to server {} \n {:?} \n\n", id, ws_stream);

      /*
       * server greet client and start listening/responding to messages
       */
      let greet_result = ws_stream
        .send(Message::text("Welcome!".to_string()))
        .await;
      if let Err(err) = greet_result {
        eprintln!("Failed to send greeting message: {}", err);
      }
      tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_stream.next().await {
          println!("{}: {:?}", id, msg);           
          if msg.is_text() || msg.is_binary() {           
            let msg_str_option = msg.as_text();
            if let Some(msg_str) = msg_str_option {
              let parse_json_result = parse_json_message(msg_str);
              match parse_json_result {
                Ok(mut value) => {
                  value["id"] = serde_json::Value::String(id.to_string());
                  println!("Do something {:?}", value);
                  let response_str = match serde_json::to_string(&value) {
                    Ok(value) => value,
                    Err(err) => {
                      eprintln!("Something went wrong creating response str: {}", err);
                      "{\"error\": true}".to_string()
                    }
                  };
                  let response_result = ws_stream
                    .send(Message::text(response_str))
                    .await;
                  if let Err(err) = response_result {
                    eprintln!("Failed to send response message: {}", err);
                  }
                },
                Err(err) => {
                  eprintln!("Error parsing json on received message: {}", err);
                }                  
              };
            } else {
              eprintln!("Err getting msg as text: {:?}", msg);
            }
          }
        }
        Ok::<_, Error>(())
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