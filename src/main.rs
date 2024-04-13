use futures_util::{SinkExt, StreamExt};
use http::Uri;
use tokio::net::TcpListener;
use tokio_websockets::{ClientBuilder, Error, Message, ServerBuilder};

#[tokio::main]
async fn main() -> Result<(), Error> {
  let listener = TcpListener::bind("127.0.0.1:3000").await?;

  tokio::spawn(async move {
    while let Ok((stream, _)) = listener.accept().await {
      println!("Bind to port");
      let mut ws_stream = ServerBuilder::new()
        .accept(stream)
        .await?;

      tokio::spawn(async move {
        println!("Socket server start");
        // Just an echo server, really
        while let Some(Ok(msg)) = ws_stream.next().await {
          println!("Server got: {:?}", msg);
          if msg.is_text() || msg.is_binary() {
            ws_stream.send(msg).await?;
          }
        }

        Ok::<_, Error>(())
      });
    }

    Ok::<_, Error>(())
  });

  client().await?;

  Ok(())
}

async fn client() -> Result<(), Error> {
  let uri = Uri::from_static("ws://127.0.0.1:3000");
  let (mut client, _) = ClientBuilder::from_uri(uri).connect().await?;

  client.send(Message::text("Hello world!")).await?;
  while let Some(Ok(msg)) = client.next().await {
    if let Some(text) = msg.as_text() {
      assert_eq!(text, "Hello world!");
      println!("Client got: {}", text);
      // We got one message, just stop now
      client.close().await?;
    }
  }
  Ok(())
}