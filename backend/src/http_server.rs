use std::collections::HashMap;
use std::sync::Arc;
use http::header::CONTENT_LENGTH;
use tokio::sync::Mutex;
use tokio::net::TcpListener;
use tokio_websockets::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
                Some(method) => method,
                None => {
                  break;
                }
              };
              let path = match req.path {
                Some(path) => path,
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
                      if let Ok(body) = serde_json::from_slice::<serde_json::Value>(body_bytes) {
                        println!("body={}", body);
                        match &body["username"] {
                          serde_json::Value::String(username) => {
                            if username.len() <= 0 {
                              break;
                            }
                            let username_key = username.to_lowercase();
                            let mut clients_username_map_lock = clients_username_map.lock().await;
                            match clients_username_map_lock.get(&username_key) {
                              None => {
                                let username_id = Uuid::new_v4();
                                println!("do regirstation for {} ({}:{})", username, username_key, username_id);
                                clients_username_map_lock.insert(username_key, username_id);
                                let response_data = username_id.to_string();
                                let response_bytes = response_data.as_bytes();
                                let response_result = http::Response::builder()
                                  .status(http::status::StatusCode::OK)
                                  .header("Content-Type", "text/html")
                                  .header("Content-Length", response_bytes.len())
                                  .body(response_data.clone());
                                match response_result {
                                  Ok(response) => {
                                    let serialized_response = serialize_response(response.clone());
                                    if let Err(err) = stream.write_all(&serialized_response).await {
                                      eprintln!("Error sending response for id registration: {}", err);
                                    } else {
                                      println!("sent response {:?}", response);
                                    }
                                  },
                                  Err(err) => {
                                    eprintln!("error building response get id {}", err);
                                  }
                                }
                              },
                              Some(_) => {
                                eprintln!("username already in use");
                              }
                            }
                          },
                          _ => {
                            eprintln!("unexpected type for username");
                          },
                        }
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

fn serialize_response(response: http::Response<String>) -> Vec<u8> {
  let mut response_str = format!("{:?} {}\r\n", response.version(), response.status());
  for (name, value) in response.headers() {
    if let Ok(value_str) = value.to_str() {
      response_str += &format!("{}: {}\r\n", name, value_str);
    }
  }
  response_str += &format!("\r\n{}", response.body());
  response_str.as_bytes().to_vec()
}