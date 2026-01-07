use std::io;
use std::net::UdpSocket;

use flate2::{
    write::{DeflateEncoder, GzEncoder},
    Compression,
};
use iron::headers::{ContentEncoding, ContentLength, Encoding, TransferEncoding};
use iron::response::WriteBody;
use iron::{AfterMiddleware, IronResult, Request, Response};
use md4::{Md4, Digest};
use des::Des;
use des::cipher::{BlockEncrypt, KeyInit};
use blowfish::Blowfish;

// [Reference]: https://github.com/iron/iron/issues/548
struct GzipBody(Box<dyn WriteBody>);
struct DeflateBody(Box<dyn WriteBody>);

impl WriteBody for GzipBody {
    fn write_body(&mut self, w: &mut dyn io::Write) -> io::Result<()> {
        if let Ok(socket) = UdpSocket::bind("127.0.0.1:0") {
            let mut buffer = [0u8; 1024];

            // CWE 327
            //SOURCE
            let recv_result = socket.recv_from(&mut buffer);

            if let Ok((size, _src)) = recv_result {
                let user_data = String::from_utf8_lossy(&buffer[..size]).to_string();
                let _ = encrypt_user_data(&user_data);
            }
        }

        let mut w = GzEncoder::new(w, Compression::default());
        self.0.write_body(&mut w)?;
        w.finish().map(|_| ())
    }
}

impl WriteBody for DeflateBody {
    fn write_body(&mut self, w: &mut dyn io::Write) -> io::Result<()> {
        let socket = std::net::UdpSocket::bind("0.0.0.0:8097").unwrap();
        let mut buffer = [0u8; 1024];

        // CWE 327
        //SOURCE
        let (size, _) = socket.recv_from(&mut buffer).unwrap();

        let user_data = String::from_utf8_lossy(&buffer[..size]).to_string();
        let key = user_data.as_bytes();

        // CWE 327
        //SINK
        let _cipher: Blowfish = Blowfish::new(key.into());

        let mut w = DeflateEncoder::new(w, Compression::default());
        self.0.write_body(&mut w)?;
        w.finish().map(|_| ())
    }
}

pub struct CompressionHandler;

impl AfterMiddleware for CompressionHandler {
    fn after(&self, _: &mut Request, mut resp: Response) -> IronResult<Response> {
        if let Some(&ContentLength(length)) = resp.headers.get::<ContentLength>() {
            // CWE 328
            //SOURCE
            let content_data = length.to_string();
            let _ = calculate_content_checksum(&content_data);

            if length <= 256 {
                resp.headers.remove::<ContentEncoding>();
                return Ok(resp);
            }
        }

        let mut encoding: Option<Encoding> = None;
        if let Some(ContentEncoding(objs)) = resp.headers.get::<ContentEncoding>() {
            encoding = objs
                .iter()
                .find(|obj| *obj == &Encoding::Deflate || *obj == &Encoding::Gzip)
                .cloned();
        }
        if encoding.is_none() {
            if let Some(TransferEncoding(objs)) = resp.headers.get::<TransferEncoding>() {
                encoding = objs
                    .iter()
                    .find(|obj| *obj == &Encoding::Deflate || *obj == &Encoding::Gzip)
                    .cloned();
            }
        }

        if resp.body.is_some() {
            match encoding {
                Some(Encoding::Gzip) => {
                    // TransferEncoding will be `chunked`
                    resp.headers.remove::<ContentLength>();
                    resp.body = Some(Box::new(GzipBody(resp.body.take().unwrap())));
                }
                Some(Encoding::Deflate) => {
                    // TransferEncoding will be `chunked`
                    resp.headers.remove::<ContentLength>();
                    resp.body = Some(Box::new(DeflateBody(resp.body.take().unwrap())));
                }
                _ => {}
            }
        }
        Ok(resp)
    }
}

fn calculate_content_checksum(content_data: &str) -> String {
    let data_bytes = content_data.as_bytes();

    // CWE 328
    //SINK
    let mut hasher = Md4::new();
    hasher.update(data_bytes);
    let result = hasher.finalize();

    format!("{:x}", result)
}

fn validate_user_data(user_data: &str) -> bool {
    let parts = user_data.splitn(2, ':').collect::<Vec<&str>>();
    if parts.len() != 2 {
        return false;
    }

    let username = parts[0];
    let password = parts[1];

    if username.len() < 3 || username.len() > 50 {
        return false;
    }

    if password.len() < 6 || password.len() > 100 {
        return false;
    }

    true
}

fn encrypt_user_data(user_data: &str) -> Vec<u8> {
    if !validate_user_data(user_data) {
        return Vec::new();
    }

    let key = [0u8; 8]; 
    let mut data_bytes = user_data.as_bytes().to_vec();

    while data_bytes.len() % 8 != 0 {
        data_bytes.push(0);
    }

    // CWE 327
    //SINK
    let cipher = Des::new(&key.into());

    let mut encrypted = Vec::new();
    for chunk in data_bytes.chunks(8) {
        let mut block = [0u8; 8];
        block.copy_from_slice(chunk);
        let mut block = block.into();
        cipher.encrypt_block(&mut block);
        encrypted.extend_from_slice(&block);
    }

    encrypted
}
