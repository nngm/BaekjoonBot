use std::{
    collections::HashMap,
    convert::TryFrom,
    io::{self, BufReader, BufWriter, prelude::*},
    net::{TcpListener, TcpStream},
    sync::Arc,
    thread,
    time::Duration,
};

use super::Method;

pub struct RequestHeader {
    pub method: Method,
    pub headers: Vec<(String, String)>,
}

pub type Handler = fn(
    req_header: RequestHeader,
    reader: BufReader<&TcpStream>,
    writer: BufWriter<&TcpStream>,
) -> io::Result<()>;

pub struct Server {
    addr: &'static str,
    routes: Arc<HashMap<String, Handler>>,
}

impl Server {
    pub fn new(addr: &'static str, routes: &[(&str, Handler)]) -> Self {
        Self {
            addr,
            routes: Arc::new(routes.iter().map(|v| (v.0.to_owned(), v.1)).collect()),
        }
    }

    pub fn run(&self) -> io::Result<()> {
        let listen = TcpListener::bind(self.addr)?;

        for stream in listen.incoming() {
            if let Ok(stream) = stream {
                let routes = self.routes.clone();
                thread::spawn(|| {
                    let mut stream = stream;
                    if stream.set_read_timeout(Some(Duration::new(30, 0))).is_err() {
                        return;
                    }

                    match Self::route(&stream, routes) {
                        Ok(_) => (),
                        Err(e) => match e.kind() {
                            io::ErrorKind::InvalidInput
                            | io::ErrorKind::InvalidData
                            | io::ErrorKind::TimedOut
                            | io::ErrorKind::WouldBlock => {
                                let _ = stream
                                    .write_all("HTTP/1.1 401 Unauthorized\r\n\r\n".as_bytes());
                            }
                            _ => (),
                        },
                    }
                });
            } else {
                continue;
            }
        }

        Err(io::Error::from(io::ErrorKind::Interrupted))
    }

    fn route(stream: &TcpStream, routes: Arc<HashMap<String, Handler>>) -> io::Result<()> {
        let mut reader = BufReader::new(stream);
        let writer = BufWriter::new(stream);

        let method: Method;
        let path: String;
        let mut headers = Vec::<(String, String)>::new();

        const INVALID_INPUT: io::ErrorKind = io::ErrorKind::InvalidInput;

        {
            let mut start_str = String::new();
            if reader.read_line(&mut start_str)? == 0 {
                return Err(INVALID_INPUT.into());
            }
            let start: Vec<&str> = start_str.split(' ').collect();

            method = Method::try_from(*(start.first().ok_or::<io::Error>(INVALID_INPUT.into())?))?;

            path = start
                .get(1)
                .ok_or::<io::Error>(INVALID_INPUT.into())?
                .to_string();
        }

        loop {
            let mut header_str = String::new();
            if reader.read_line(&mut header_str)? == 0 {
                return Err(INVALID_INPUT.into());
            } else if header_str == "\r\n" {
                break;
            }

            let mut split = header_str.split(": ");
            let key = split.next().ok_or::<io::Error>(INVALID_INPUT.into())?;
            let mut value = split.next().ok_or::<io::Error>(INVALID_INPUT.into())?;

            let value_len = value.len();
            if &value[value_len - 2..value_len] != "\r\n" {
                return Err(INVALID_INPUT.into());
            }
            value = &value[0..value_len - 2];

            headers.push((key.to_string(), value.to_string()));
        }

        if let Some(handler) = routes.get(path.as_str()) {
            handler(RequestHeader { method, headers }, reader, writer)
        } else {
            Err(INVALID_INPUT.into())
        }
    }
}

pub fn read_content(
    req_header: &RequestHeader,
    reader: &mut BufReader<&TcpStream>,
) -> io::Result<Vec<u8>> {
    const INVALID_INPUT: io::ErrorKind = io::ErrorKind::InvalidInput;

    let content_len: usize;
    if let Some(header) = req_header.headers.iter().find(|&v| v.0 == "Content-Length") {
        content_len = header
            .1
            .parse()
            .or::<io::Error>(Err(INVALID_INPUT.into()))?;
    } else {
        return Err(INVALID_INPUT.into());
    }

    let mut content = vec![0; content_len];
    if content_len > 0 {
        reader.read_exact(content.as_mut_slice())?;
    }

    Ok(content)
}
