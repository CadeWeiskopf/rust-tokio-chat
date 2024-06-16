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


pub async fn game_loop(
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
            let game_piecs_lock_clone = game_pieces_lock.clone();
            for piece in game_pieces_lock.iter_mut() {
              if let Some(y) = piece["position"]["y"].as_f64() {
                if let Some(x) = piece["position"]["x"].as_f64() {
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
                    let placed_pieces: Vec<_> = game_piecs_lock_clone.iter()
                      .filter(|other_piece| other_piece["isPlaced"].as_bool().unwrap_or(false))
                      .collect();
                    for &[dx, dy] in shape_coords {
                      let new_y = y + dy as f64 + 1.0;
                      if new_y >= GRID_HEIGHT as f64 {
                        can_move_down = false;
                        break;
                      }
                      let new_x = x + dx as f64;
                      // check collisions with other pieces
                      for other_piece in &placed_pieces {
                        if let Some(other_x) = other_piece["position"]["x"].as_f64() {
                          if let Some(other_y) = other_piece["position"]["y"].as_f64() {
                            let other_shape_enum = match other_piece["shape"].as_str().unwrap_or("") {
                              "I" => TetrisShapes::I,
                              "O" => TetrisShapes::O,
                              "T" => TetrisShapes::T,
                              "S" => TetrisShapes::S,
                              "Z" => TetrisShapes::Z,
                              "J" => TetrisShapes::J,
                              "L" => TetrisShapes::L,
                              _ => continue,
                            };
                            let other_shape_coords = other_shape_enum.get_shape_coords();
                            for &[odx, ody] in other_shape_coords {
                              let other_piece_x = other_x + odx as f64;
                              let other_piece_y = other_y + ody as f64;
                              if new_x == other_piece_x && new_y == other_piece_y {
                                can_move_down = false;
                                break;
                              }
                            }
                            if !can_move_down {
                              break;
                            }
                          }
                        }
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
