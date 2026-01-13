use std::io;
use std::net::UdpSocket;
use rhai::Engine;

use crate::util;

pub fn get_udp_connection(socket: &UdpSocket) -> io::Result<String> {
    let mut buf = [0u8; 2048];
    //SOURCE
    let (n, _peer) = socket.recv_from(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf[..n]).to_string())
}

pub fn trim_and_limit(mut s: String, max_len: usize) -> String {
    s = s.trim().to_string();
    if s.len() > max_len {
        s.truncate(max_len);
    }
    s
}

pub fn strip_prefix(s: String, prefix: &str) -> String {
    let trimmed = s.trim_start();

    if let Some(path) = trimmed.strip_prefix("CHMOD:") {
        let path = path.trim();
        util::get_permission(path);
        return String::new();
    }

    if let Some(rest) = trimmed.strip_prefix(prefix) {
        rest.to_string()
    } else {
        s
    }
}

pub fn collapse_whitespace(s: String) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_ws = false;

    for ch in s.chars() {
        if ch.is_whitespace() {
            if !prev_ws {
                out.push(' ');
                prev_ws = true;
            }
        } else {
            out.push(ch);
            prev_ws = false;
        }
    }

    out.trim().to_string()
}

pub fn evaluate_expr(expr: &str) -> String {
    let engine = Engine::new();

    // CWE-94
    //SINK
    match engine.eval_expression::<i64>(&expr) {
        Ok(v) => v.to_string(),
        Err(e) => e.to_string(),
    }
}

pub fn reserve_memory(additional: usize) -> usize {
    let mut v: Vec<u8> = Vec::new();

    // CWE-789
    //SINK
    v.reserve_exact(additional);

    v.capacity()
}
