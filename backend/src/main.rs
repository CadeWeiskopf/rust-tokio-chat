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

  web_socket_server::start_web_socket_server(clients_map).await?;

  http_server::start_server().await?;

  loop {
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
  }
}