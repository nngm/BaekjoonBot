use serde_json::{self, json};
use std::{
    io::{self, BufReader, BufWriter, prelude::*},
    net::TcpStream,
};

use crate::http::{self, Method, server::RequestHeader};

fn respond_ok_json(
    body_json: &serde_json::Value,
    writer: &mut BufWriter<&TcpStream>,
) -> io::Result<()> {
    let v = serde_json::to_vec(body_json)?;
    let body = v.as_slice();
    writer.write_all(
        format!(
            "HTTP/1.1 200 OK\r\n\
            User-Agent: DiscordBot (https://baekjoonbot.hexa.pro, 2.0)\r\n\
            Content-Type: application/json\r\n\
            Content-Length: {}\r\n\
            \r\n",
            body.len()
        )
        .as_bytes(),
    )?;
    writer.write_all(body)?;
    writer.flush()?;
    Ok(())
}

const INTERACTION_PING: i64 = 1;

fn pong(writer: &mut BufWriter<&TcpStream>) -> io::Result<()> {
    let body = json!({"type":1});
    respond_ok_json(&body, writer)
}

pub fn interactions(
    req_header: RequestHeader,
    mut reader: BufReader<&TcpStream>,
    mut writer: BufWriter<&TcpStream>,
) -> io::Result<()> {
    const INVALID_INPUT: io::ErrorKind = io::ErrorKind::InvalidInput;
    const INVALID_DATA: io::ErrorKind = io::ErrorKind::InvalidData;

    match req_header.method {
        Method::Post => (),
        _ => {
            return Err(INVALID_INPUT.into());
        }
    }

    let content: serde_json::Value =
        serde_json::from_slice(http::server::read_content(&req_header, &mut reader)?.as_slice())?;
    let interaction_type = content
        .get("type")
        .ok_or::<io::Error>(INVALID_DATA.into())?
        .as_i64()
        .ok_or::<io::Error>(INVALID_DATA.into())?;

    match interaction_type {
        INTERACTION_PING => pong(&mut writer),
        _ => Err(INVALID_DATA.into()),
    }
}
