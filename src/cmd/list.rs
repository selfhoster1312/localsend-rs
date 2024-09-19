use argh::FromArgs;

#[derive(Clone, Debug, FromArgs)]
#[argh(subcommand, name = "list")]
/// list other devices found on LAN
pub struct ListCmd {}
