use ring::{error::Unspecified, signature};
use serde_json::{self, json};
use std::{
    io::{self, BufReader, BufWriter, prelude::*},
    net::TcpStream,
};

use crate::http::{self, Method, server::RequestHeader};
use super::ENV;

fn parse_hex_str(string: &str) -> Option<Vec<u8>> {
    let len = string.len();
    if len % 2 != 0 {
        return None;
    }

    let mut vec = Vec::with_capacity(len / 2);

    for i in string.as_bytes().windows(2).step_by(2) {
        vec.push(u8::from_str_radix(str::from_utf8(i).ok()?, 16).ok()?);
    }

    Some(vec)
}

fn verify_interaction(req_header: &RequestHeader, body: &[u8]) -> Result<(), Unspecified> {
    let sig = parse_hex_str(
        req_header
            .headers
            .iter()
            .find(|&v| v.0 == "X-Signature-Ed25519")
            .ok_or(Unspecified)?
            .1
            .as_str(),
    )
    .ok_or(Unspecified)?;

    let time = req_header
        .headers
        .iter()
        .find(|&v| v.0 == "X-Signature-Timestamp")
        .ok_or(Unspecified)?
        .1
        .as_str();

    let message = [time.as_bytes(), body].concat();

    let key = signature::UnparsedPublicKey::new(
        &signature::ED25519,
        parse_hex_str(ENV.pubkey)
            .ok_or(Unspecified)?,
    );

    key.verify(&message, &sig)
}

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

    let body = http::server::read_content(&req_header, &mut reader)?;
    verify_interaction(&req_header, body.as_slice()).or::<io::Error>(Err(INVALID_INPUT.into()))?;

    let object: serde_json::Value = serde_json::from_slice(body.as_slice())?;
    let interaction_type = object
        .get("type")
        .ok_or::<io::Error>(INVALID_DATA.into())?
        .as_i64()
        .ok_or::<io::Error>(INVALID_DATA.into())?;

    match interaction_type {
        INTERACTION_PING => pong(&mut writer),
        _ => {
            Err(INVALID_DATA.into())
        }
    }
}
