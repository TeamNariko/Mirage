mod config;
mod format;
mod protocol;

use std::{error::Error, net::TcpListener};

fn main() -> Result<(), Box<dyn Error>> {
    let config = config::load(std::env::current_exe()?.with_file_name("mirage.json"))?;
    let bind = config.bind.clone();
    let messages = config::Messages::from_config(config)?;
    let listener = TcpListener::bind(&bind)?;

    println!("Mirage {}", env!("CARGO_PKG_VERSION"));
    println!("Listening on {bind}");
    for stream in listener.incoming().flatten() {
        let _ = protocol::handle_connection(stream, &messages);
    }
    Ok(())
}
