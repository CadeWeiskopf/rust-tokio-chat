use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_websockets::{Error, Message, ServerBuilder};
use uuid::Uuid;
use serde_json::Value;
use crate::data::{User, parse_json_message};

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
          break
        }
      };
      let (tx, rx) = ws_stream.split();

      // create new user in clients_map
      let id = Uuid::new_v4();
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
      let clients_usernames_map_clone = clients_usernames_map.clone();
      tokio::spawn(async move {
        let client_rx_stream_clone = client_rx_stream.clone();
        loop {
          let mut client_rx_lock = client_rx_stream_clone.lock().await;
          match client_rx_lock.next().await {
            Some(Ok(msg)) => {
              match msg.as_text() {
                Some(msg_str) => {
                  match parse_json_message(msg_str) {
                    Ok(mut msg_json) => {
                      let init_obj = msg_json.get("init");
                      match init_obj {
                          Some(init_data) => {
                            println!("do registration process {:?}", init_data);
                            match init_data {
                              Value::Object(init_data_obj) => {
                                let username_registration_id_opt = init_data_obj.get("id");
                                let username_registration_id = match username_registration_id_opt {
                                  Some(username_registration_id) => {
                                    match username_registration_id {
                                      serde_json::Value::String(username_registration_id) => {
                                        username_registration_id
                                      }, 
                                      _ => {
                                        eprintln!("bad data type for username registration id");
                                        continue;
                                      }
                                    };
                                    username_registration_id
                                  },
                                  None => {
                                    eprintln!("no username registration id in init data obj");
                                    continue;
                                  }
                                };
                                let username = init_data_obj.get("username");
                                match username {
                                  Some(username) => {
                                    match username {
                                      Value::String(username) => {
                                        let username_lowercase = username.to_lowercase();
                                        let username_key = username_lowercase.trim();
                                        let username_len = username_key.len();
                                        if username_len <= 0 {
                                          eprintln!("username is not acceptable len");
                                          continue;
                                        }
                                        let mut client_usernames_lock = clients_usernames_map_clone.lock().await;
                                        let registered_username_id = client_usernames_lock.get(username_key);
                                        match registered_username_id {
                                          Some(id_to_check_against) => {
                                            if format!("\"{}\"", id_to_check_against) == username_registration_id.to_string() {
                                              println!("finish registration for user {} {}=={}", username, id_to_check_against, username_registration_id);
                                              client_usernames_lock.insert((&username_key).to_string(), id);
                                              println!("clients usernames : {:?}", client_usernames_lock);
                                            } else {
                                              eprintln!("username provided id does not match provided: {}, want: {}", username_registration_id, id_to_check_against);
                                            }
                                          },
                                          None => {
                                            eprintln!("no registration registration for user {}", username);
                                          }
                                        }
                                        std::mem::drop(client_usernames_lock);
                                      }, 
                                      _ => {
                                        eprintln!("unexpected username type");
                                        break;
                                      }
                                    }
                                  },
                                  None => {
                                    eprintln!("username not in init data obj");
                                    break;
                                  }
                                }
                              },
                              _ => {
                                eprintln!("init data not expected type");
                              }
                            }
                          },
                          None => {
                            // send message to chat
                            msg_json["key"] = Value::String(Uuid::new_v4().to_string());
                            let mut messages_vec_lock = msg_vec_clone.lock().await;
                            messages_vec_lock.push(msg_json);
                            std::mem::drop(messages_vec_lock);
                          },
                      }
                    },
                    Err(err) => {
                      eprintln!("Error parsing message as json: {} \n=====!!!\n", err);
                    }
                  }   
                },
                None => {
                  eprintln!("None message as text, likely disconnecting \n=====\n");
                }
              }
              let client_tx_lock = client_tx_stream.lock().await;
              std::mem::drop(client_tx_lock);
            },
            Some(Err(err)) => {
              eprintln!("Error receiving message: {} \n=====!!!\n", err);
            },
            None => {
              let mut clients_map_remove_lock = clients_map_remove_clone.lock().await;
              clients_map_remove_lock.remove(&id);
              // let mut clients_username_map_lock = clients_usernames_map.lock().await;
              // clients_username_map_lock.remove(new_user.name_key);
              println!("\n {} disconnected, \n Total Clients = {}\n=====\n", id, clients_map_remove_lock.len());
              std::mem::drop(clients_map_remove_lock);
              break;
            }
          }
          std::mem::drop(client_rx_lock);
        }
      });
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