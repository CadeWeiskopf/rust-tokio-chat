use std::collections::HashMap;
use std::sync::Arc;
use http::header::HOST;
use tokio::sync::Mutex;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_websockets::{Error, Message, ServerBuilder};
use uuid::Uuid;
use serde_json::Value;
use url::Url;
use crate::data::{User, parse_json_message};
use rand::{thread_rng, Rng};

pub async fn start_web_socket_server(
  clients_map: Arc<Mutex<HashMap<Uuid, Arc<Mutex<User>>>>>,
  clients_usernames_map: Arc<Mutex<HashMap<String, Uuid>>>
) -> Result<(), Error> {
  let messages_vec: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));

  let connection_handler_clients_map = clients_map.clone();
  let connection_handler_messages_vec = messages_vec.clone();
  client_connection_handler(
    connection_handler_clients_map, 
    connection_handler_messages_vec, 
    clients_usernames_map
  ).await?;

  let dispatch_clients_map = clients_map.clone();
  let dispatch_messages_vec = messages_vec.clone();
  dispatch_messages_loop(dispatch_clients_map, dispatch_messages_vec).await?;

  let game_clients_map = clients_map.clone();
  game_loop(game_clients_map).await?;

  Ok::<_, Error>(())
}

async fn client_connection_handler(
  clients_map: Arc<Mutex<HashMap<Uuid, Arc<Mutex<User>>>>>, 
  messages_vec: Arc<Mutex<Vec<Value>>>,
  clients_usernames_map: Arc<Mutex<HashMap<String, Uuid>>>
) -> Result<(), Error> {
  let listener = TcpListener::bind("127.0.0.1:8080").await?;  
  tokio::spawn(async move {
    println!("Web socket server listening... {:?}", listener); 
    while let Ok((stream, _addr)) = listener.accept().await {
      let mut buffer = [0; 1024];
      let mut new_client_id_option = None;
      if let Ok(bytes_read) = stream.peek(&mut buffer).await {
        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("====\nreq: {}", request);
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);
        match req.parse(request.as_bytes()) {
            Ok(httparse::Status::Complete(_req_body_start)) => {
              let path = match req.path {
                Some(path) => path,
                None => {
                  eprintln!("no path for ws");
                  continue;
                }
              };
              let mut host_value = None;
              for h in req.headers {
                if h.name == HOST {
                  host_value = Some(h.value);
                  break;
                }
              }
              match host_value {
                Some(host) => {
                  // register client from url params
                  let host_name = String::from_utf8_lossy(host);
                  let parsed_url = Url::parse(&format!("ws://{}{}", host_name, path));
                  match parsed_url {
                    Ok(url_data) => {
                      let mut username = None;
                      let mut username_id = None;
                      for (key, value) in url_data.query_pairs() {
                        if key == "username" {
                          username = Some(value);
                        } else if key == "id" {
                          username_id = Some(value);
                        }
                      }
                      if let (
                        Some(username_value), 
                        Some(username_id_value)
                      ) = (
                        username, 
                        username_id
                      ) {
                        println!("register: {} {}", username_value, username_id_value);
                        let clients_usersname_map_lock = 
                          clients_usernames_map.lock().await;
                        let username_value_lowercase = username_value.to_lowercase();
                        if let Some(usernames_registration_id) = clients_usersname_map_lock.get(&username_value_lowercase) {
                          println!("user in map : {}", usernames_registration_id);
                          if username_id_value != usernames_registration_id.to_string() {
                            eprintln!("id does not match the usernames_registration_id");
                            continue;
                          }
                          new_client_id_option = Some(Uuid::new_v4());
                          // clients_usersname_map_lock.insert(username_value_lowercase, id);
                        } else {
                          eprintln!("could not find in usernames map");
                          continue;
                        }
                      } else {
                        eprintln!("connection does not have the necessary url query params");
                        continue;
                      }
                    },
                    Err(err) => {
                      eprintln!("Error parsing host_name {}", err);
                    }
                  }
                },
                None => {
                  eprintln!("no Host header");
                  continue;
                }
              }
            }
            Ok(httparse::Status::Partial) => {
              println!("Incomplete request, need more data in ws connect");
            }
            Err(err) => {
              eprintln!("Error parsing request on ws connect: {}", err);
            }
        }
      }

      if let Some(id) = new_client_id_option {
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
            continue;
          }
        };
        let (tx, rx) = ws_stream.split();

        // create new user in clients_map
        // let id = Uuid::new_v4();
        let client_rx_stream = Arc::new(Mutex::new(rx));
        let client_tx_stream = Arc::new(Mutex::new(tx));
        let mut clients_map_lock = clients_map.lock().await;
        let new_user = User {
          tx_stream: client_tx_stream.clone(),
          rx_stream: client_rx_stream.clone(),
          name: "tbd".to_string(),
          name_key: "tab".to_string()
        };
        clients_map_lock.insert(id, Arc::new(Mutex::new(new_user)));
        println!("\nNew connection to server {} \n Total Clients = {}\n====\n", id, clients_map_lock.len());
        std::mem::drop(clients_map_lock);
        
        // listen for client messages
        // push client's messages to messages_vec
        let msg_vec_clone = messages_vec.clone();
        let clients_map_remove_clone = clients_map.clone();
        let rate_limiter = Arc::new(tokio_utils::RateLimiter::new(std::time::Duration::from_millis(500)));     
        tokio::spawn(async move {
          let client_rx_stream_clone = client_rx_stream.clone();
          loop {
            let mut client_rx_lock = client_rx_stream_clone.lock().await;
            rate_limiter.throttle(|| async {
              match client_rx_lock.next().await {
                Some(Ok(msg)) => {
                  match msg.as_text() {
                    Some(msg_str) => {
                      match parse_json_message(msg_str) {
                        Ok(mut msg_json) => {
                          // send message to chat
                          msg_json["key"] = Value::String(Uuid::new_v4().to_string());
                          let mut messages_vec_lock = msg_vec_clone.lock().await;
                          messages_vec_lock.push(msg_json);
                          std::mem::drop(messages_vec_lock);
                        },
                        Err(err) => {
                          eprintln!("Error parsing message as json: {} \n=====!!!\n", err);
                        }
                      }   
                    },
                    None => {
                      let mut clients_map_remove_lock = clients_map_remove_clone.lock().await;
                      clients_map_remove_lock.remove(&id);
                      println!("\n {} disconnected, \n Total Clients = {}\n=====\n", id, clients_map_remove_lock.len());
                      std::mem::drop(clients_map_remove_lock);
                    }
                  }
                },
                Some(Err(err)) => {
                  eprintln!("Error receiving message: {} \n=====!!!\n", err);
                  let mut clients_map_remove_lock = clients_map_remove_clone.lock().await;
                  clients_map_remove_lock.remove(&id);
                  println!("\n {} disconnected, \n Total Clients = {}\n=====\n", id, clients_map_remove_lock.len());
                  std::mem::drop(clients_map_remove_lock);
                  return;
                },
                None => {}
              }
              std::mem::drop(client_rx_lock);
            }).await;
            
          }
        });
      } else {
        eprintln!("no id set to new_client_id_option")
      }
    }
    Ok::<_, Error>(())
  });
  Ok::<_, Error>(())
}

async fn dispatch_messages_loop(
  clients_map: Arc<Mutex<HashMap<Uuid, Arc<Mutex<User>>>>>, 
  messages_vec: Arc<Mutex<Vec<Value>>>
) -> Result<(), Error> {
  tokio::spawn(async move {
    loop {
      tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
      let mut messages_vec_lock = messages_vec.lock().await;
      let n = messages_vec_lock.len();
      for msg_json in messages_vec_lock.drain(0..n) {
        let clients_map_lock = clients_map.lock().await;
        for (id, user) in clients_map_lock.iter() {
          println!("{} sent: {:?}\n====\n", id, msg_json);
          let u = user.lock().await;
          let mut u_tx_stream = u.tx_stream.lock().await;
          let send_msg_result = u_tx_stream.send(Message::text(msg_json.to_string())).await;
          match send_msg_result {
            Ok(()) => {},
            Err(err) => {
              eprintln!("err sending message to client {}: {}", id, err);
            }
          }
          std::mem::drop(u_tx_stream);
          std::mem::drop(u);
        }
        std::mem::drop(clients_map_lock);
      }
    }  
  });
  Ok::<_, Error>(())
}

enum TetrisShapes {
  I,
  O,
  T,
  S,
  Z,
  J,
  L,
}
impl TetrisShapes {
  fn as_str(&self) -> &'static str {
    match self {
      TetrisShapes::I => "I",
      TetrisShapes::O => "O",
      TetrisShapes::T => "T",
      TetrisShapes::S => "S",
      TetrisShapes::Z => "Z",
      TetrisShapes::J => "J",
      TetrisShapes::L => "L",
    }
  }
}
fn get_random_tetris_shape() -> TetrisShapes {
  let index = thread_rng().gen_range(0..=6);
  match index {
    0 => TetrisShapes::I,
    1 => TetrisShapes::O,
    2 => TetrisShapes::T,
    3 => TetrisShapes::S,
    4 => TetrisShapes::Z,
    5 => TetrisShapes::J,
    6 => TetrisShapes::L,
    _ => unreachable!(),
  }
}

struct TetrisPiece {
  is_stopped: bool,
  shape: TetrisShapes  
}

enum GamePhase {
  Pregame,
  Starting,
  Live
}
impl GamePhase {
  fn as_str(&self) -> &'static str {
    match self {
      GamePhase::Pregame => "pregame",
      GamePhase::Starting => "starting",
      GamePhase::Live => "live",
    }
  }
}

struct GameState {
  active_pieces: Vec<TetrisPiece>,
  settled_pieces: Vec<TetrisPiece>,
  game_phase: GamePhase,
  init_time: std::time::Instant,
  start_time: Option<std::time::Instant>,
  live_time: Option<std::time::Instant>
}

async fn game_loop(
  clients_map: Arc<Mutex<HashMap<Uuid, Arc<Mutex<User>>>>>, 
) -> Result<(), Error> {
  let pregame_time_limit = std::time::Duration::from_secs(10);
  let mut game_state = GameState {
    active_pieces: Vec::new(),
    settled_pieces: Vec::new(),
    game_phase: GamePhase::Pregame,
    init_time: std::time::Instant::now(),
    start_time: None,
    live_time: None,
  };
  tokio::spawn(async move {
    loop {
      tokio::time::sleep(tokio::time::Duration::from_millis(17)).await;
      let clients_map_lock = clients_map.lock().await;
      for (id, user) in clients_map_lock.iter() {
        // println!("game state: {}", game_state.game_phase.as_str());
        match game_state.game_phase {
          GamePhase::Pregame => {
            println!("pregaming");
            if game_state.init_time.elapsed() >= pregame_time_limit {
              println!("make live!");
              game_state.game_phase = GamePhase::Starting;
            }
          },
          GamePhase::Starting => {
            println!("starting...");
            if let Some(start_time) = game_state.start_time {
              if start_time.elapsed() >= pregame_time_limit {
                game_state.game_phase = GamePhase::Live;
              }
            } else {
              game_state.start_time = Some(std::time::Instant::now());
            }
          },
          GamePhase::Live => {
            println!("game is now live");
          }
        }
      }
      std::mem::drop(clients_map_lock);
    }  
  });
  Ok::<_, Error>(())
}