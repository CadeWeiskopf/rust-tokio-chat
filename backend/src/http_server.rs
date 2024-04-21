use tokio::net::TcpListener;
use tokio_websockets::Error;
use tokio::io::AsyncReadExt;

pub async fn start_server() -> Result<(), Error> {
  let http_listener = TcpListener::bind("127.0.0.1:8081").await?;
  tokio::spawn(async move {
    println!("HTTP server listening... {:?}", http_listener);
    while let Ok((mut stream, _)) = http_listener.accept().await {
      let mut buffer = [0; 1024];
      if let Ok(bytes_read) = stream.read(&mut buffer).await {
        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);
        match req.parse(request.as_bytes()) {
            Ok(httparse::Status::Complete(_)) => {
              let method = req.method.unwrap();
              let path = req.path.unwrap();
              if method == "GET" && path == "/id" {
                println!("gen id ");
              }
            }
            Ok(httparse::Status::Partial) => {
              println!("Incomplete request, need more data");
            }
            Err(err) => {
              eprintln!("Error parsing request: {}", err);
            }
        }
      }
    } 
  });
  Ok::<_, Error>(())
}