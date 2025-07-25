use std::{
    io::{self, BufWriter, prelude::*},
    net::TcpStream,
    sync::Arc,
    time::Duration,
};

use rustls;
use webpki_roots;

use super::Method;

pub fn https_request(
    method: Method,
    url: &'static str,
    path: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> io::Result<Vec<u8>> {
    const INVALID_INPUT: io::ErrorKind = io::ErrorKind::InvalidInput;

    let root_store = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.into(),
    };
    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    let url_with_port = {
        let mut url = url.to_owned();
        url.push_str(":443");
        url
    };

    let mut conn = rustls::ClientConnection::new(
        Arc::new(config),
        url.try_into().or::<io::Error>(Err(INVALID_INPUT.into()))?,
    )
    .or::<io::Error>(Err(INVALID_INPUT.into()))?;
    let mut sock = TcpStream::connect(url_with_port.as_str())?;
    sock.set_read_timeout(Some(Duration::new(30, 0)))?;

    let mut tls = rustls::Stream::new(&mut conn, &mut sock);
    {
        let mut writer = BufWriter::new(&mut tls);
        writer.write_all(
            [
                method.into(),
                b" " as &[u8],
                path.as_bytes(),
                b" HTTP/1.1\r\n",
            ]
            .concat()
            .as_slice(),
        )?;

        writer.write_all(["Host: ", url, "\r\n"].concat().as_bytes())?;
        writer.write_all(b"Connection: close\r\n")?;

        for header in headers {
            writer.write_all(
                [header.0.as_bytes(), b": ", header.1.as_bytes(), b"\r\n"]
                    .concat()
                    .as_slice(),
            )?;
        }

        writer.write_all(b"\r\n")?;
        writer.write_all(body)?;
        writer.flush()?;
    }

    let mut plaintext = Vec::new();
    tls.read_to_end(&mut plaintext)?;
    Ok(plaintext)
}
