use std::env;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

fn main() {
    let url = env::args()
        .nth(1)
        .unwrap_or_else(|| "http://127.0.0.1:8080/healthz".to_string());

    if let Err(err) = probe(&url) {
        eprintln!("healthcheck failed: {err}");
        std::process::exit(1);
    }
}

fn probe(url: &str) -> io::Result<()> {
    let (host, port, path) = parse_http_url(url)?;

    let mut stream = TcpStream::connect((host.as_str(), port))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;

    let request = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
        path, host
    );
    stream.write_all(request.as_bytes())?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    if !response.starts_with("HTTP/1.1 200") {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "unexpected status line: {}",
                response.lines().next().unwrap_or("")
            ),
        ));
    }

    if !response.contains("\r\nok") && !response.ends_with("ok") && !response.contains("\nok") {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "response body missing ok",
        ));
    }

    Ok(())
}

fn parse_http_url(url: &str) -> io::Result<(String, u16, String)> {
    const PREFIX: &str = "http://";
    if !url.starts_with(PREFIX) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "only http:// URLs are supported",
        ));
    }

    let remainder = &url[PREFIX.len()..];
    let mut parts = remainder.splitn(2, '/');
    let host_port = parts.next().unwrap_or("");
    if host_port.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "missing host in URL",
        ));
    }

    let path_part = parts.next().unwrap_or("");
    let mut path = format!("/{}", path_part);
    while path.len() > 1 && path.ends_with('/') {
        path.pop();
    }
    if path.is_empty() {
        path.push('/');
    }

    let mut host_parts = host_port.splitn(2, ':');
    let host = host_parts.next().unwrap().to_string();
    let port = host_parts
        .next()
        .map(|p| {
            p.parse()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid port"))
        })
        .transpose()?
        .unwrap_or(80);

    Ok((host, port, path))
}
