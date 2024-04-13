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

  loop {
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
  }

  Ok(())
}
