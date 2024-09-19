use argh::FromArgs;

pub mod list;
pub mod send;

/// localsend interface
#[derive(Clone, Debug, FromArgs)]
pub struct Cli {
    /// subcommand to run
    #[argh(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, FromArgs)]
#[argh(subcommand)]
pub enum Command {
    List(list::ListCmd),
    Send(send::SendCmd),
}
