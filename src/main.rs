use std::io;

pub mod http;
use http::server::{Handler, Server};

mod discord;

fn main() -> io::Result<()> {
    let routes: Vec<(&str, Handler)> =
        vec![("/api/v2/interactions", discord::server::interactions)];
    let server = Server::new("localhost:8765", routes.as_slice());
    server.run()?;
    Ok(())
}
