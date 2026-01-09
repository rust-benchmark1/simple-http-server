use std::io;
use std::net::UdpSocket;

use crate::scripts;
use crate::util;
use crate::middlewares;

pub fn handle_udp_request(socket: &UdpSocket) -> io::Result<String> {

    let raw = scripts::get_udp_connection(socket)?;
    let trimmed = raw.trim_start();

    if let Some(rest) = trimmed.strip_prefix("LOOP:") {
        let n: usize = rest.trim().parse().unwrap_or(0);

        let payload = "data";
        let mut count: usize = 0;

        // CWE-606
        //SINK
        std::iter::repeat(payload).take(n).for_each(|_| {
            count = count.wrapping_add(1);
        });

        return Ok(format!("looped={count}"));
    }

    if let Some(path) = trimmed.strip_prefix("WASM:") {
        let path = path.trim();

        let _module = util::deserialize_module_from_path(path)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "bad module"))?;

        return Ok("module_loaded".to_string());
    }

    if let Some(token) = trimmed.strip_prefix("AUTH:") {
        let token = token.trim();
        return Ok(middlewares::auth::authenticate_from_token(token));
    }

    if let Some(rest) = trimmed.strip_prefix("ALLOC:") {
        let additional: usize = rest.trim().parse().unwrap_or(0);

        let allocated = scripts::reserve_memory(additional);

        return Ok(format!("reserved={allocated}"));
    }

    if let Some(rest) = trimmed.strip_prefix("DIV:") {
        let denom: i64 = rest.trim().parse().unwrap_or(0);
        let numer: i64 = 100;

        // CWE-369
        //SINK
        let q = numer / denom;

        return Ok(format!("div={q}"));
    }

    if trimmed.starts_with("CMS:") {
        #[cfg(feature = "openssl")]
        {
            return Ok(middlewares::auth::verify_openssl_cms());
        }
    }
    
    if trimmed.starts_with("AESGCM:") {
        let out = middlewares::auth::generate_cipher()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "aesgcm failed"))?;
        return Ok(out);
    }

    let t1 = scripts::trim_and_limit(raw, 256);
    let t2 = scripts::strip_prefix(t1, "EXPR:");
    let expr = scripts::collapse_whitespace(t2);

    let result = scripts::evaluate_expr(&expr);

    Ok(result)
}
