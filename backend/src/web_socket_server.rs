use std::error;
use std::str::FromStr;
use std::{borrow::Borrow, collections::HashMap};
use std::sync::Arc;
use futures_util::future::join_all;
use http::header::HOST;
use tokio::sync::Mutex;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_websockets::{Error, Message, ServerBuilder};
use uuid::Uuid;
use serde_json::json;
use serde_json::Value;
use url::Url;
use crate::data::{User, parse_json_message, MessageTypes};
use rand::{thread_rng, Rng};

pub async fn start_web_socket_server(
  clients_map: Arc<Mutex<HashMap<Uuid, Arc<Mutex<User>>>>>,
  clients_usernames_map: Arc<Mutex<HashMap<String, Uuid>>>
) -> Result<(), Error> {
  let messages_vec: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
  let game_messages_vec: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));

  let connection_handler_clients_map = clients_map.clone();
  let connection_handler_messages_vec = messages_vec.clone();
  client_connection_handler(
    connection_handler_clients_map, 
    connection_handler_messages_vec, 
    clients_usernames_map
  ).await?;

  let dispatch_clients_map = clients_map.clone();
  let dispatch_messages_vec = messages_vec.clone();
  dispatch_messages_loop(
    dispatch_clients_map, 
    dispatch_messages_vec
  ).await?;

  // let game_clients_map = clients_map.clone();
  // game_loop(game_clients_map, game_messages_vec).await?;

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
      let mut new_username: Option<String> = None;
      if let Ok(bytes_read) = stream.peek(&mut buffer).await {
        let request = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("====\nreq: {}", request);
        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);

        // on connect do the client register
        // get url params username and id
        // will use these values to lookup and permit the user entry
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
                      
                      let username_value_lowercase = username_value.to_lowercase();
                      let clients_usersname_map_lock = 
                        clients_usernames_map.lock().await;
                      if let Some(usernames_registration_id) = clients_usersname_map_lock.get(&username_value_lowercase) {
                        println!("user in map : {}", usernames_registration_id);
                        new_client_id_option = Some(usernames_registration_id.clone());
                        new_username = Some(username_value.to_string());
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

      // if new_client_id_option, user successfully was registered
      // set up the client stream
      if let Some(id) = new_client_id_option {
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
        let client_rx_stream = Arc::new(Mutex::new(rx));
        let client_tx_stream = Arc::new(Mutex::new(tx));
        let mut clients_map_lock = clients_map.lock().await;
        let username = match new_username {
          Some(ref v) => v,
          _ => {continue}
        };
        let new_user = User {
          tx_stream: client_tx_stream.clone(),
          rx_stream: client_rx_stream.clone(),
          name: username.to_string(),
          name_key: "tab".to_string(),
          matched_with: None,
          active_requests: Arc::new(Mutex::new(Vec::new()))
        };
        let new_user_arc_mtx = Arc::new(Mutex::new(new_user));
        clients_map_lock.insert(id, new_user_arc_mtx.clone());
        
        // let mut clients_usersname_map_lock = clients_usernames_map.lock().await;
        // let username_value_lowercase = username_value.to_lowercase();
        // clients_usersname_map_lock.insert(username_value_lowercase, *id);
        // std::mem::drop(clients_usersname_map_lock);  
        println!("\nNew connection to server {} \n Total Clients = {}\n====\n", id, clients_map_lock.len());
        std::mem::drop(clients_map_lock);
        
        // listen for client messages
        let msg_vec_clone = messages_vec.clone();
        let clients_map_remove_clone = clients_map.clone();
        let clients_username_clone = clients_usernames_map.clone();
        let rate_limiter = Arc::new(tokio_utils::RateLimiter::new(std::time::Duration::from_millis(500)));     
        tokio::spawn(async move {
          let client_rx_stream_clone = client_rx_stream.clone();
          let mut is_disconnected = false;
          loop {
            let mut client_rx_lock = client_rx_stream_clone.lock().await;
            rate_limiter.throttle(|| async {
              match client_rx_lock.next().await {
                Some(Ok(msg)) => {
                  match msg.as_text() {
                    Some(msg_str) => {
                      match parse_json_message(msg_str) {
                        Ok(mut msg_json) => {
                          match &msg_json["matchRequest"] {
                            serde_json::Value::String(username_to_challenge) => {
                              println!("locking");
                              let client_username_map_lock = clients_username_clone.lock().await;
                              if let Some(user_id) = client_username_map_lock.get(&username_to_challenge.to_lowercase()) {
                                println!("got match_request: {:?} {:?} from {:?}", username_to_challenge, user_id, id);
                                if user_id == &id {
                                  // user cant challenge themself
                                  return;
                                }
                                let client_map_lock: tokio::sync::MutexGuard<HashMap<Uuid, Arc<Mutex<User>>>> = clients_map_remove_clone.lock().await;
                                if let Some(user) = client_map_lock.get(user_id) {
                                  println!("got user");
                                  let user_to_challenge_lock = user.lock().await;
                                  let mut user_tx_lock = user_to_challenge_lock.tx_stream.lock().await;
                                  let challenge_message_json = json!({
                                    "type": MessageTypes::MatchRequest.as_u8(),
                                    "requestFrom": {
                                      "id": id.to_string(),
                                      "name": new_username
                                    }
                                  });
                                  user_tx_lock.send(Message::text(challenge_message_json.to_string())).await;
                                  println!("sent messag");
                                  if let Some(user_sending_request) = client_map_lock.get(&id) {
                                    let user_sending_lock = user_sending_request.lock().await;
                                    let mut user_requests = user_sending_lock.active_requests.lock().await;
                                    user_requests.push(*user_id);
                                    println!("added to reqs : {:?}", user_requests);
                                    std::mem::drop(user_requests);
                                    std::mem::drop(user_sending_lock);
                                  }
                                  std::mem::drop(user_tx_lock);
                                  std::mem::drop(user_to_challenge_lock);
                                } else {
                                  eprintln!(" didnnt find user in map {:?}", client_map_lock.keys());
                                }
                                std::mem::drop(client_map_lock);
                              } else {
                                eprintln!("didnt find the user {:?} ", username_to_challenge);
                              }
                              std::mem::drop(client_username_map_lock);
                            }
                            _ => {
                              match &msg_json["matchAccept"] {
                                serde_json::Value::String(user_id_to_accept) => {
                                  let client_map_lock: tokio::sync::MutexGuard<HashMap<Uuid, Arc<Mutex<User>>>> = clients_map_remove_clone.lock().await;
                                  if let Ok(user_uuid) = Uuid::from_str(user_id_to_accept) {
                                    if let Some(user_to_accept) = client_map_lock.get(&user_uuid) {
                                      let user_lock = user_to_accept.lock().await;
                                      println!("user's active reqs: {:?}, req from: {:?}", user_lock.active_requests, id);
                                      let mut active_requests_lock = user_lock.active_requests.lock().await;
                                      if active_requests_lock.contains(&id) {
                                        println!("should accept");
                                        let local_user_lock = new_user_arc_mtx.lock().await;
                                        let mut local_active_reqs_lock = local_user_lock.active_requests.lock().await;
                                        local_active_reqs_lock.clear();
                                        std::mem::drop(local_active_reqs_lock);
                                        std::mem::drop(local_user_lock);
                                        active_requests_lock.clear();
                                        let mut clients_in_match: HashMap<Uuid, Arc<Mutex<User>>> = HashMap::new();
                                        clients_in_match.insert(user_uuid, user_to_accept.clone());
                                        clients_in_match.insert(id, new_user_arc_mtx.clone());
                                        let clients_in_match_mtx = Arc::new(Mutex::new(clients_in_match));
                                        game_loop(clients_in_match_mtx).await;
                                      }
                                      std::mem::drop(active_requests_lock);
                                      std::mem::drop(user_lock);
                                          
                                    }
                                  } else {
                                    eprintln!("err converting match accept id to uuid");
                                  }
                                  std::mem::drop(client_map_lock);
                                },
                                _ => {
                                  // send message to chat
                                  msg_json["key"] = Value::String(Uuid::new_v4().to_string());
                                  let mut messages_vec_lock = msg_vec_clone.lock().await;
                                  messages_vec_lock.push(msg_json);
                                  std::mem::drop(messages_vec_lock);   
                                }
                              }
                            }
                          }
                        },
                        Err(err) => {
                          eprintln!("Error parsing message as json: {} \n=====!!!\n", err);
                        }
                      }   
                    },
                    None => {
                      let mut clients_map_remove_lock = clients_map_remove_clone.lock().await;
                      let mut clients_usernames_map_lock = clients_username_clone.lock().await;
                      if let Some(u) = &new_username {
                        clients_usernames_map_lock.remove(&u.to_lowercase());
                      }
                      clients_map_remove_lock.remove(&id);
                      println!("\n {} disconnected, \n Total Clients = {}\n=====\n", id, clients_map_remove_lock.len());
                      is_disconnected = true;
                      std::mem::drop(clients_map_remove_lock);
                      std::mem::drop(clients_usernames_map_lock);
                    }
                  }
                },
                Some(Err(err)) => {
                  eprintln!("Error receiving message: {} \n=====!!!\n", err);
                  let mut clients_map_remove_lock = clients_map_remove_clone.lock().await;
                  let mut clients_usernames_map_lock = clients_username_clone.lock().await;
                  if let Some(u) = &new_username {
                    clients_usernames_map_lock.remove(&u.to_lowercase());
                  }
                  clients_map_remove_lock.remove(&id);
                  println!("\n {} disconnected, \n Total Clients = {}\n=====\n", id, clients_map_remove_lock.len());
                  is_disconnected = true;
                  std::mem::drop(clients_map_remove_lock);
                  std::mem::drop(clients_usernames_map_lock);
                  return;
                },
                None => {}
              }
              std::mem::drop(client_rx_lock);
            }).await;
            if is_disconnected {
              break
            }
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
  
  fn get_shape_coords(&self) -> &'static [[i32; 2]] {
    match self {
      TetrisShapes::I => &[[0, 0], [1, 0], [2, 0], [3, 0]],
      TetrisShapes::O => &[[0, 0], [1, 0], [0, 1], [1, 1]],
      TetrisShapes::T => &[[1, 0], [0, 1], [1, 1], [2, 1]],
      TetrisShapes::S => &[[1, 0], [2, 0], [0, 1], [1, 1]],
      TetrisShapes::Z => &[[0, 0], [1, 0], [1, 1], [2, 1]],
      TetrisShapes::J => &[[0, 0], [0, 1], [1, 1], [2, 1]],
      TetrisShapes::L => &[[2, 0], [0, 1], [1, 1], [2, 1]],
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
  clients_map: Arc<Mutex<HashMap<Uuid, Arc<Mutex<User>>>>>
) -> Result<(), Error> {
  println!("game loop"); 
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
    let clients_map_lock = clients_map.lock().await;
    let user_keys_futures = clients_map_lock.clone().into_iter().map(|(key,value)| {
      async move {
        json!({
          "key": key.to_string(),
          "name": value.lock().await.name
        })
      }
    });
    let user_keys = join_all(user_keys_futures).await;
    let send_futures = clients_map_lock.clone().into_iter().map(|(_uuid, user_arc_mtx)| {
      let user_keys_clone = user_keys.clone();
      async move {
        let user_lock = user_arc_mtx.lock().await;
        let mut user_tx_lock = user_lock.tx_stream.lock().await;
        let message_to_send = json!({
          "type": MessageTypes::MatchStart.as_u8(),
          "users": user_keys_clone
        });
        user_tx_lock.send(Message::text(message_to_send.to_string())).await;
        std::mem::drop(user_tx_lock);
        std::mem::drop(user_lock);
      }
    });
    std::mem::drop(clients_map_lock);
    join_all(send_futures).await;
    loop {
      tokio::time::sleep(tokio::time::Duration::from_millis(17)).await;
      match game_state.game_phase {
        GamePhase::Pregame => {
          if game_state.init_time.elapsed() >= pregame_time_limit {
            println!("pregame -> starting");
            game_state.game_phase = GamePhase::Starting;
          }
        },
        GamePhase::Starting => {
          if let Some(start_time) = game_state.start_time {
            if start_time.elapsed() >= pregame_time_limit {
              println!("starting -> live");
              game_state.game_phase = GamePhase::Live;
            }
          } else {
            game_state.start_time = Some(std::time::Instant::now());
          }
        },
        GamePhase::Live => {
          let fps = 32;
          let rate = 1000.0 / (fps as f32);
          const GRAVITY: f32 = 0.5;
          let mut update_interval = tokio::time::interval(std::time::Duration::from_millis(rate as u64));
          let mut game_pieces: Arc<Mutex<Vec<Value>>> = Arc::new(Mutex::new(Vec::new()));
          const GRID_WIDTH: u8 = 20;
          const GRID_HEIGHT: u8 = 24;
          loop {
            update_interval.tick().await;
            // println!("update tick");
            
            // dispatch tick update
            let clients_map_lock = clients_map.lock().await;
            let mut game_pieces_lock = game_pieces.lock().await;
            for piece in game_pieces_lock.iter_mut() {
              if let Some(y) = piece["position"]["y"].as_f64() {
                // let shape_coords = piece["shape"].ge
                if let Some(shape_str) = piece["shape"].as_str() {
                  let shape_enum = match shape_str {
                    "I" => TetrisShapes::I,
                    "O" => TetrisShapes::O,
                    "T" => TetrisShapes::T,
                    "S" => TetrisShapes::S,
                    "Z" => TetrisShapes::Z,
                    "J" => TetrisShapes::J,
                    "L" => TetrisShapes::L,
                    _ => {
                      eprintln!("shape is not a shape");
                      continue;
                    }
                  };
                  let shape_coords = shape_enum.get_shape_coords();
                  let mut can_move_down = true;
                  for &[dx, dy] in shape_coords {
                    let new_y = y + dy as f64 + 1.0;
                    if new_y >= GRID_HEIGHT as f64 {
                      can_move_down = false;
                      break;
                    }
                  }
                  if can_move_down {
                    piece["position"]["y"] = json!(y as f64 + GRAVITY as f64);
                  } else {
                    piece["isPlaced"] = json!(true);
                  }
                }
              }
            }
            for (i, (uuid, user_arc_mtx)) in clients_map_lock.iter().enumerate() {
              let tetris_shape = get_random_tetris_shape();
              let exists = game_pieces_lock.iter().any(|piece| {
                piece["owner"] == uuid.to_string() && piece["isPlaced"] == false
              });
              if !exists {
                game_pieces_lock.push(json!({
                  "owner": uuid.to_string(),
                  "shape": tetris_shape.as_str(),
                  "position": {
                    "x": if i % 2 != 0 { GRID_WIDTH / 2 } else { 0 },
                    "y": 0
                  },
                  "isPlaced": false
                }));
              }
            }
            let send_futures = clients_map_lock.iter().map(|(uuid, user_arc_mtx)| {
              let game_piece_lock_clone = game_pieces_lock.clone();
              async move {
                let tetris_shape = get_random_tetris_shape();
                let msg = json!({
                  "type": MessageTypes::MatchUpdate.as_u8(),
                  "gamePieces": serde_json::Value::from_iter(game_piece_lock_clone.into_iter())
                });
                let user_lock = user_arc_mtx.lock().await;
                let mut user_tx_lock = user_lock.tx_stream.lock().await;
                let send_result = user_tx_lock.send(Message::text(msg.to_string())).await;
                if let Err(send_err) = send_result {
                  // err sending, possible disconnect
                  println!("Disconnected");
                }
                std::mem::drop(user_tx_lock);
                std::mem::drop(user_lock);
              }
            });
            join_all(send_futures).await;
            std::mem::drop(clients_map_lock);
          }
        }
      }
    }  
  });
  Ok::<_, Error>(())
}