# localsend-rs

A Rust crate for interacting with [LocalSend](https://localsend.org/) clients according to the [LocalSend protocol](https://github.com/localsend/protocol). This crate implements the LocalSend v2.1 protocol.

## How to use

- configure the client identity by creating a `localsend::info::Info` instance; usually `let info = Info::from_xdg().await.unwrap();`
- initialize a new LocalSend instance with that device info: `LocalSend::new(info)`

## License

GNU aGPLv3
