use std::collections::HashMap;
use std::sync::Arc;
use http::header::CONTENT_LENGTH;
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use tokio_websockets::Error;
use tokio::io::AsyncReadExt;
use uuid::Uuid;

pub async fn start_server(clients_username_map: Arc<Mutex<HashMap<String, Uuid>>>) -> Result<(), Error> {
  let http_listener = TcpListener::bind("127.0.0.1:8081").await?;
  tokio::spawn(async move {
    println!("HTTP server listening... {:?}", http_listener);
    while let Ok((mut stream, _)) = http_listener.accept().await {
      let mut buffer = [0; 1024];
      if let Ok(bytes_read) = stream.read(&mut buffer).await {
        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("req: {}", request);
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);
        match req.parse(request.as_bytes()) {
            Ok(httparse::Status::Complete(req_body_start)) => {
              let method = match req.method {
                Some(m) => m,
                None => {
                  break;
                }
              };
              let path = match req.path {
                Some(p) => p,
                None => {
                  break;
                }
              };
              if method == "POST" && path == "/id" {
                  for header in req.headers {
                    if header.name == CONTENT_LENGTH {
                      let str_value = std::str::from_utf8(header.value).expect("Invalid UTF-8 sequence");
                      let num: usize = str_value.trim().parse().expect("Failed to parse usize");
                      let body_bytes = &request.as_bytes()[req_body_start..req_body_start + num];
                      let body_str = std::str::from_utf8(body_bytes).expect("Invalid UTF-8 sequence");
                      if let Ok(body) = serde_json::from_slice::<serde_json::Value>(body_bytes) {
                        println!("body={}", body);
                      }
                      break;
                    }
                  };
                
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