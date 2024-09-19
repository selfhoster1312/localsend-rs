use localsend::LocalSend;
use std::fs::File;
use std::io::Read;

mod cmd;
use cmd::{Cli, Command};

#[tokio::main]
async fn main() {
    let cmd: Cli = argh::from_env();

    let localsend = LocalSend::new().await.unwrap();
    localsend.send_announce().await.unwrap();

    match cmd.command {
        Command::List(_) => {
            unimplemented!();
        },
        Command::Send(send_cmd) => {
            let mut file = File::open(send_cmd.file).unwrap();
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).unwrap();
            // TODO: mimetype
            localsend.send(&send_cmd.receiver, "text/plain", buf).await.unwrap();
        }
    }
}
