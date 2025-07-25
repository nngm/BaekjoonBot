use serde_json::{self, json};
use std::io;

use super::ENV;
use crate::http::{Method, client::https_request};

pub fn register_hello() -> io::Result<()> {
    let path = format!("/api/v10/applications/{}/commands", ENV.appid);
    let auth = format!("Bot {}", ENV.token);
    let body = serde_json::to_vec(&json!({"name":"hello", "description":"Hello, World!"}))
        .or::<io::Error>(Err(io::ErrorKind::InvalidInput.into()))?;
    let len = (body.len()).to_string();
    let headers = vec![
        ("Authorization", auth.as_str()),
        (
            "User-Agent",
            "DiscordBot (https://baekjoonbot.hexa.pro, 2.0)",
        ),
        ("Content-Type", "application/json"),
        ("Content-Length", len.as_str()),
    ];

    let result = https_request(
        Method::Post,
        "discord.com",
        path.as_str(),
        headers.as_slice(),
        body.as_slice(),
    )?;

    if &result[..15] == b"HTTP/1.1 200 OK" {
        Ok(())
    } else {
        Err(io::ErrorKind::Other.into())
    }
}
