mod http_server;
mod web_socket_server;
mod data;

use tokio_websockets::Error;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::data::User;

#[tokio::main]
async fn main() -> Result<(), Error> {
  let clients: HashMap<Uuid, Arc<Mutex<User>>> = HashMap::new();
  let clients_map = Arc::new(Mutex::new(clients));
  let clients_usernames: HashMap<String, Uuid> = HashMap::new();
  let clients_username_map = Arc::new(Mutex::new(clients_usernames));
  let clients_username_ws_clone = clients_username_map.clone();
  
  web_socket_server::start_web_socket_server(clients_map, clients_username_ws_clone).await?;

  http_server::start_server(clients_username_map).await?;

  loop {
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
  }
}