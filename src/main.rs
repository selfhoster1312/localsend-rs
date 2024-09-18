use localsend::LocalSend;
use std::fs::File;
use std::io::Read;

#[tokio::main]
async fn main() {
    let mut args = std::env::args();
    args.next().unwrap();

    let localsend = LocalSend::new().await.unwrap();
    localsend.send_announce().await.unwrap();
    for arg in args {
        let mut file = File::open(arg).unwrap();
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).unwrap();
        localsend.send("Secret Papaya", "text/plain", buf).await.unwrap();
    }
}
