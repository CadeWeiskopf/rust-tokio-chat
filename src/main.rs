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
        Ok(stream) => stream,
        Err(err) => {
          eprintln!("Failed to accept WebSocket connection: {}", err);
          break;
        }
      };
      let id = Uuid::new_v4();
      println!("New connection to server {}", id);

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
          if msg.is_text() || msg.is_binary() {
            println!("{}: {:?}", id, msg);           
            let response_result = ws_stream
              .send(msg)
              .await;
            if let Err(err) = response_result {
              eprintln!("Failed to send response message: {}", err);
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
