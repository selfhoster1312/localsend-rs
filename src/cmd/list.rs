use argh::FromArgs;
use tokio::net::UdpSocket;

use localsend::info::Info;

#[derive(Clone, Debug, FromArgs)]
#[argh(subcommand, name = "list")]
/// list other devices found on LAN
pub struct ListCmd {}

pub async fn wait_for_lan() {
    let socket = UdpSocket::bind("0.0.0.0:53317").await.unwrap();
    // let _ = socket.set_read_timeout(Some(timeout));
    // let _ = socket.set_write_timeout(Some(timeout));
    let _ = socket.set_broadcast(true);

    let mut buf = [0; 4096];
    while let Ok(size) = socket.recv(&mut buf).await {
        if let Ok(response) = serde_json::from_slice::<Info>(&buf[0..size]) {
            println!(
                "Received LAN advertisement for LocalSend client: {}",
                response.config.alias
            );
        }
    }
}
