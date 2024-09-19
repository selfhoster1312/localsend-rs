use argh::FromArgs;

use std::path::PathBuf;

#[derive(Clone, Debug, FromArgs)]
#[argh(subcommand, name = "send")]
/// Send a file to another device
pub struct SendCmd {
    /// destination for the file
    #[argh(positional)]
    pub receiver: String,
    /// file to be sent
    #[argh(positional)]
    pub file: PathBuf,
}
