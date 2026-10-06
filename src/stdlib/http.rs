//! Client HTTP/1.1 minimal de la bibliothèque standard Kastel.
//!
//! Cette couche repose directement sur `std::net::TcpStream` pour rester sans
//! dépendance externe. Elle couvre les requêtes HTTP/1.1 classiques sur `http://`.
//! HTTPS n'est volontairement pas exposé ici : il nécessite une couche TLS
//! dédiée qui sera traitée séparément.
//!
//! Surface publique :
//! - `http_get(url)`
//! - `http_request(method, url, headers, body)`
//!
//! La réponse est un `Record` :
//! `{ status, headers, body, version, reason }`.
//! `body` reste un `List<int>` afin de ne pas corrompre les réponses binaires.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use crate::{
    error::runtime_error::RuntimeError,
    runtime::{object::Object, value::Value},
};

const MAX_HEADERS: usize = 64 * 1024;
const MAX_BODY: usize = 16 * 1024 * 1024;
const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug)]
struct ParsedUrl {
    host: String,
    port: u16,
    path: String,
    host_header: String,
}

fn http_module_error(message: impl Into<String>) -> RuntimeError {
    RuntimeError::ModuleError(format!("http: {}", message.into()))
}

fn expect_string(value: &Value) -> Result<String, RuntimeError> {
    value.as_string_value().ok_or(RuntimeError::TypeError)
}

fn parse_port(text: &str) -> Result<u16, RuntimeError> {
    text.parse::<u16>()
        .map_err(|_| http_module_error("invalid port"))
}

fn parse_url(value: &Value) -> Result<ParsedUrl, RuntimeError> {
    let url = expect_string(value)?;
    let Some(rest) = url.strip_prefix("http://") else {
        if url.starts_with("https://") {
            return Err(http_module_error(
                "HTTPS is not supported by std.http; use a TLS-enabled library",
            ));
        }
        return Err(http_module_error("URL must start with http://"));
    };

    let authority_end = rest
        .find(|character| character == '/' || character == '?' || character == '#')
        .unwrap_or(rest.len());
    let authority = &rest[..authority_end];
    let suffix = &rest[authority_end..];

    if authority.is_empty() || authority.contains('@') {
        return Err(http_module_error("invalid URL authority"));
    }

    let (host, port) = if authority.starts_with('[') {
        let Some(end) = authority.find(']') else {
            return Err(http_module_error("invalid IPv6 host"));
        };
        let host = &authority[1..end];
        if host.is_empty() {
            return Err(http_module_error("empty host"));
        }
        let remainder = &authority[end + 1..];
        let port = if remainder.is_empty() {
            80
        } else if let Some(port_text) = remainder.strip_prefix(':') {
            if port_text.is_empty() {
                return Err(http_module_error("invalid port"));
            }
            parse_port(port_text)?
        } else {
            return Err(http_module_error("invalid IPv6 authority"));
        };
        (host.to_string(), port)
    } else if let Some((host, port_text)) = authority.rsplit_once(':') {
        if host.is_empty() || port_text.is_empty() || host.contains(':') {
            return Err(http_module_error("invalid host or port; IPv6 hosts must use brackets"));
        }
        (host.to_string(), parse_port(port_text)?)
    } else {
        (authority.to_string(), 80)
    };

    if host.chars().any(|character| character.is_ascii_whitespace()) {
        return Err(http_module_error("host cannot contain whitespace"));
    }

    if suffix.contains('#') {
        return Err(http_module_error("URL fragments are not sent in HTTP requests"));
    }

    let path = if suffix.is_empty() {
        "/".to_string()
    } else if suffix.starts_with('?') {
        format!("/{suffix}")
    } else {
        suffix.to_string()
    };

    let host_header = if port == 80 {
        if host.contains(':') {
            format!("[{host}]")
        } else {
            host.clone()
        }
    } else if host.contains(':') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    };

    Ok(ParsedUrl {
        host,
        port,
        path,
        host_header,
    })
}

fn is_http_token_byte(byte: u8) -> bool {
    matches!(
        byte,
        b'0'..=b'9'
            | b'a'..=b'z'
            | b'A'..=b'Z'
            | b'!'
            | b'#'
            | b'$'
            | b'%'
            | b'&'
            | b'\''
            | b'*'
            | b'+'
            | b'-'
            | b'.'
            | b'^'
            | b'_'
            | b'`'
            | b'|'
            | b'~'
    )
}

fn validate_header_name(name: &str) -> Result<(), RuntimeError> {
    if name.is_empty() || !name.bytes().all(is_http_token_byte) {
        return Err(http_module_error("invalid header name"));
    }
    Ok(())
}

fn validate_header_value(value: &str) -> Result<(), RuntimeError> {
    if value.contains('\r') || value.contains('\n') {
        return Err(http_module_error("header value cannot contain CR or LF"));
    }
    Ok(())
}

fn expect_headers(value: &Value) -> Result<Vec<(String, String)>, RuntimeError> {
    let Value::Object(handle) = value else {
        return Err(RuntimeError::TypeError);
    };

    let object = handle.borrow();
    let Object::Dict(entries) = &*object else {
        return Err(RuntimeError::TypeError);
    };

    entries
        .iter()
        .map(|(name, value)| {
            let name = expect_string(name)?;
            let value = expect_string(value)?;
            validate_header_name(&name)?;
            validate_header_value(&value)?;
            Ok((name, value))
        })
        .collect()
}

fn expect_body(value: &Value) -> Result<Vec<u8>, RuntimeError> {
    match value {
        Value::None => Ok(Vec::new()),
        Value::Object(handle) => {
            let object = handle.borrow();
            match &*object {
                Object::String(text) => Ok(text.as_bytes().to_vec()),
                Object::Array(elements) | Object::Tuple(elements) => elements
                    .iter()
                    .map(|value| match value {
                        Value::Integer(byte) if (0..=255).contains(byte) => Ok(*byte as u8),
                        _ => Err(RuntimeError::TypeError),
                    })
                    .collect(),
                _ => Err(RuntimeError::TypeError),
            }
        }
        _ => Err(RuntimeError::TypeError),
    }
}

fn body_value(bytes: Vec<u8>) -> Value {
    Value::new_array(
        bytes
            .into_iter()
            .map(|byte| Value::Integer(i64::from(byte)))
            .collect(),
    )
}

fn header_contains(headers: &[(String, String)], target: &str) -> bool {
    headers
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case(target))
}

fn header_value<'a>(headers: &'a [(String, String)], target: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(target))
        .map(|(_, value)| value.as_str())
}

fn write_request(
    stream: &mut TcpStream,
    method: &str,
    url: &ParsedUrl,
    user_headers: &[(String, String)],
    body: &[u8],
) -> Result<(), RuntimeError> {
    if method.is_empty() || !method.bytes().all(is_http_token_byte) {
        return Err(http_module_error("invalid HTTP method"));
    }

    if header_contains(user_headers, "connection") {
        return Err(http_module_error(
            "the Connection header is reserved; std.http uses Connection: close",
        ));
    }

    if let Some(content_length) = header_value(user_headers, "content-length") {
        let parsed = content_length
            .parse::<usize>()
            .map_err(|_| http_module_error("invalid Content-Length"))?;
        if parsed != body.len() {
            return Err(http_module_error(
                "Content-Length does not match request body size",
            ));
        }
    }

    write!(stream, "{method} {} HTTP/1.1\r\n", url.path)
        .map_err(|error| RuntimeError::ModuleError(format!("http: write request: {error}")))?;

    if !header_contains(user_headers, "host") {
        write!(stream, "Host: {}\r\n", url.host_header).map_err(|error| {
            RuntimeError::ModuleError(format!("http: write request: {error}"))
        })?;
    }

    if !header_contains(user_headers, "user-agent") {
        stream
            .write_all(b"User-Agent: Kastel/0.1.0\r\n")
            .map_err(|error| RuntimeError::ModuleError(format!("http: write request: {error}")))?;
    }

    if !header_contains(user_headers, "accept") {
        stream
            .write_all(b"Accept: */*\r\n")
            .map_err(|error| RuntimeError::ModuleError(format!("http: write request: {error}")))?;
    }

    for (name, value) in user_headers {
        write!(stream, "{name}: {value}\r\n").map_err(|error| {
            RuntimeError::ModuleError(format!("http: write request: {error}"))
        })?;
    }

    if !header_contains(user_headers, "content-length") {
        write!(stream, "Content-Length: {}\r\n", body.len()).map_err(|error| {
            RuntimeError::ModuleError(format!("http: write request: {error}"))
        })?;
    }

    stream
        .write_all(b"Connection: close\r\n\r\n")
        .map_err(|error| RuntimeError::ModuleError(format!("http: write request: {error}")))?;

    if !body.is_empty() {
        stream
            .write_all(body)
            .map_err(|error| RuntimeError::ModuleError(format!("http: write body: {error}")))?;
    }

    Ok(())
}

fn read_header_line(reader: &mut BufReader<TcpStream>, header_size: &mut usize) -> Result<Vec<u8>, RuntimeError> {
    let mut line = Vec::new();
    let read = reader
        .read_until(b'\n', &mut line)
        .map_err(|error| RuntimeError::ModuleError(format!("http: read response: {error}")))?;

    if read == 0 {
        return Err(http_module_error("unexpected end of response headers"));
    }

    *header_size = header_size.saturating_add(read);
    if *header_size > MAX_HEADERS {
        return Err(http_module_error("response headers exceed 64 KiB"));
    }

    Ok(line)
}

fn trim_http_line(mut line: Vec<u8>) -> Vec<u8> {
    while matches!(line.last(), Some(b'\r' | b'\n')) {
        line.pop();
    }
    line
}

fn read_response_headers(
    reader: &mut BufReader<TcpStream>,
) -> Result<(String, i64, String, Vec<(String, String)>), RuntimeError> {
    loop {
        let mut header_size = 0usize;
        let status_line = trim_http_line(read_header_line(reader, &mut header_size)?);
        let status_line = String::from_utf8(status_line)
            .map_err(|_| http_module_error("response status line is not valid ASCII/UTF-8"))?;
        let mut parts = status_line.splitn(3, ' ');
        let version = parts.next().unwrap_or_default();
        let code = parts
            .next()
            .ok_or_else(|| http_module_error("invalid HTTP status line"))?
            .parse::<i64>()
            .map_err(|_| http_module_error("invalid HTTP status code"))?;
        let reason = parts.next().unwrap_or_default().trim().to_string();

        if version != "HTTP/1.0" && version != "HTTP/1.1" {
            return Err(http_module_error("unsupported HTTP version"));
        }
        if !(100..=599).contains(&code) {
            return Err(http_module_error("HTTP status code out of range"));
        }

        let mut headers: Vec<(String, String)> = Vec::new();
        loop {
            let line = trim_http_line(read_header_line(reader, &mut header_size)?);
            if line.is_empty() {
                break;
            }

            let Some(colon) = line.iter().position(|byte| *byte == b':') else {
                return Err(http_module_error("invalid response header"));
            };

            let name = String::from_utf8(line[..colon].to_vec())
                .map_err(|_| http_module_error("response header name is not UTF-8"))?;
            validate_header_name(&name)?;
            let value = String::from_utf8_lossy(&line[colon + 1..]).trim().to_string();
            validate_header_value(&value)?;

            if let Some((_, existing)) = headers
                .iter_mut()
                .find(|(existing_name, _)| existing_name.eq_ignore_ascii_case(&name))
            {
                existing.push_str(", ");
                existing.push_str(&value);
            } else {
                headers.push((name.to_ascii_lowercase(), value));
            }
        }

        // Les réponses 1xx (hors mise à niveau de protocole) précèdent parfois
        // la vraie réponse finale. Elles n'ont pas de corps à retourner ici.
        if (100..200).contains(&code) && code != 101 {
            continue;
        }

        return Ok((version.to_string(), code, reason, headers));
    }
}

fn read_exact_body(reader: &mut BufReader<TcpStream>, length: usize) -> Result<Vec<u8>, RuntimeError> {
    if length > MAX_BODY {
        return Err(http_module_error("response body exceeds 16 MiB"));
    }
    let mut body = vec![0u8; length];
    reader
        .read_exact(&mut body)
        .map_err(|error| RuntimeError::ModuleError(format!("http: read body: {error}")))?;
    Ok(body)
}

fn read_chunked_body(reader: &mut BufReader<TcpStream>) -> Result<Vec<u8>, RuntimeError> {
    let mut body = Vec::new();

    loop {
        let mut line_size = 0usize;
        let line = trim_http_line(read_header_line(reader, &mut line_size)?);
        let line = String::from_utf8(line).map_err(|_| http_module_error("invalid chunk size"))?;
        let size_text = line.split(';').next().unwrap_or_default().trim();
        let size = usize::from_str_radix(size_text, 16)
            .map_err(|_| http_module_error("invalid chunk size"))?;

        if size == 0 {
            loop {
                let line = trim_http_line(read_header_line(reader, &mut line_size)?);
                if line.is_empty() {
                    break;
                }
            }
            break;
        }

        if size > MAX_BODY || body.len().saturating_add(size) > MAX_BODY {
            return Err(http_module_error("chunked response body exceeds 16 MiB"));
        }

        let mut chunk = vec![0u8; size];
        reader
            .read_exact(&mut chunk)
            .map_err(|error| RuntimeError::ModuleError(format!("http: read chunk: {error}")))?;
        body.extend_from_slice(&chunk);

        let mut suffix = [0u8; 2];
        reader.read_exact(&mut suffix).map_err(|error| {
            RuntimeError::ModuleError(format!("http: read chunk terminator: {error}"))
        })?;
        if suffix != *b"\r\n" {
            return Err(http_module_error("invalid chunk terminator"));
        }
    }

    Ok(body)
}

fn read_until_eof(reader: &mut BufReader<TcpStream>) -> Result<Vec<u8>, RuntimeError> {
    let mut body = Vec::new();
    let mut chunk = [0u8; 8192];

    loop {
        let read = reader
            .read(&mut chunk)
            .map_err(|error| RuntimeError::ModuleError(format!("http: read body: {error}")))?;
        if read == 0 {
            break;
        }
        if body.len().saturating_add(read) > MAX_BODY {
            return Err(http_module_error("response body exceeds 16 MiB"));
        }
        body.extend_from_slice(&chunk[..read]);
    }

    Ok(body)
}

fn response_value(
    version: String,
    status: i64,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
    reason: String,
) -> Value {
    let headers = headers
        .into_iter()
        .map(|(name, value)| (Value::new_string(name), Value::new_string(value)))
        .collect();

    Value::new_record(vec![
        ("status".into(), Value::Integer(status)),
        ("headers".into(), Value::new_dict(headers)),
        ("body".into(), body_value(body)),
        ("version".into(), Value::new_string(version)),
        ("reason".into(), Value::new_string(reason)),
    ])
}

fn request_impl(
    method: &str,
    url_value: &Value,
    headers_value: &Value,
    body_value: &Value,
) -> Result<Value, RuntimeError> {
    let url = parse_url(url_value)?;
    let headers = expect_headers(headers_value)?;
    let body = expect_body(body_value)?;

    let mut stream = TcpStream::connect((&*url.host, url.port)).map_err(|error| {
        RuntimeError::NetworkError {
            operation: "http_connect",
            kind: match error.kind() {
                std::io::ErrorKind::ConnectionRefused => "ConnectionRefused",
                std::io::ErrorKind::TimedOut => "TimedOut",
                std::io::ErrorKind::AddrNotAvailable => "AddrNotAvailable",
                _ => "Other",
            },
            message: error.to_string(),
        }
    })?;
    stream
        .set_read_timeout(Some(HTTP_TIMEOUT))
        .map_err(|error| RuntimeError::ModuleError(format!("http: set read timeout: {error}")))?;
    stream
        .set_write_timeout(Some(HTTP_TIMEOUT))
        .map_err(|error| RuntimeError::ModuleError(format!("http: set write timeout: {error}")))?;

    write_request(&mut stream, method, &url, &headers, &body)?;
    stream
        .flush()
        .map_err(|error| RuntimeError::ModuleError(format!("http: flush request: {error}")))?;

    let mut reader = BufReader::new(stream);
    let (version, status, reason, headers) = read_response_headers(&mut reader)?;

    let chunked = header_value(&headers, "transfer-encoding")
        .map(|value| value.split(',').any(|part| part.trim().eq_ignore_ascii_case("chunked")))
        .unwrap_or(false);

    let body = if chunked {
        read_chunked_body(&mut reader)?
    } else if let Some(content_length) = header_value(&headers, "content-length") {
        let length = content_length
            .parse::<usize>()
            .map_err(|_| http_module_error("invalid response Content-Length"))?;
        read_exact_body(&mut reader, length)?
    } else if status == 204 || status == 304 || (100..200).contains(&status) {
        Vec::new()
    } else {
        read_until_eof(&mut reader)?
    };

    Ok(response_value(version, status, headers, body, reason))
}

pub fn native_http_get(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let headers = Value::new_dict(Vec::new());
    let body = Value::None;
    request_impl("GET", &args[0], &headers, &body)
}

pub fn native_http_request(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 4 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 4,
            found: args.len(),
        });
    }

    let method = expect_string(&args[0])?;
    request_impl(&method, &args[1], &args[2], &args[3])
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("http_get".into(), Value::NativeFunction(native_http_get));
    globals.insert("http_request".into(), Value::NativeFunction(native_http_request));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::thread;

    fn string(value: &str) -> Value {
        Value::new_string(value.to_string())
    }

    fn int(value: i64) -> Value {
        Value::Integer(value)
    }

    fn headers(entries: &[(&str, &str)]) -> Value {
        Value::new_dict(
            entries
                .iter()
                .map(|(name, value)| (string(name), string(value)))
                .collect(),
        )
    }

    fn response_fields(value: &Value) -> Vec<(String, Value)> {
        value.record_fields().expect("HTTP response doit être un record")
    }

    fn field(value: &Value, name: &str) -> Value {
        response_fields(value)
            .into_iter()
            .find(|(field, _)| field == name)
            .map(|(_, value)| value)
            .unwrap_or_else(|| panic!("champ HTTP absent: {name}"))
    }

    #[test]
    fn http_get_rejects_invalid_arguments_and_urls() {
        assert!(matches!(native_http_get(&[]), Err(RuntimeError::WrongArgumentCount { .. })));
        assert!(matches!(native_http_get(&[string("https://example.com")]), Err(RuntimeError::ModuleError(_))));
        assert!(matches!(native_http_get(&[string("example.com")]), Err(RuntimeError::ModuleError(_))));
        assert!(matches!(native_http_get(&[string("http://::1/")]), Err(RuntimeError::ModuleError(_))));
        assert!(matches!(native_http_request(&[string("GET"), string("http://127.0.0.1")]), Err(RuntimeError::WrongArgumentCount { .. })));
    }

    #[test]
    fn http_request_validates_headers_and_body_bytes() {
        let url = string("http://127.0.0.1:1/");
        let bad_header_name = headers(&[("bad name", "value")]);
        assert!(matches!(
            native_http_request(&[string("GET"), url.clone(), bad_header_name, Value::None]),
            Err(RuntimeError::ModuleError(_))
        ));

        let bad_header_value = headers(&[("X-Test", "ok\r\nInjected: yes")]);
        assert!(matches!(
            native_http_request(&[string("GET"), url.clone(), bad_header_value, Value::None]),
            Err(RuntimeError::ModuleError(_))
        ));

        let bad_bytes = Value::new_array(vec![int(256)]);
        assert!(matches!(
            native_http_request(&[string("GET"), url, headers(&[]), bad_bytes]),
            Err(RuntimeError::TypeError)
        ));
    }

    #[test]
    fn http_get_parses_content_length_response() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("listener");
        let address = listener.local_addr().expect("address");

        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut request = Vec::new();
            let mut buffer = [0u8; 1024];
            loop {
                let read = stream.read(&mut buffer).expect("request read");
                request.extend_from_slice(&buffer[..read]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            let request = String::from_utf8_lossy(&request);
            assert!(request.starts_with("GET /hello?x=1 HTTP/1.1\r\n"));
            assert!(request.to_ascii_lowercase().contains("host:"));
            assert!(request.to_ascii_lowercase().contains("connection: close"));

            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nX-Test: one\r\nContent-Length: 5\r\nConnection: close\r\n\r\nhello")
                .expect("response write");
        });

        let url = string(&format!("http://{}/hello?x=1", address));
        let response = native_http_get(&[url]).expect("http_get");
        assert_eq!(field(&response, "status"), Value::Integer(200));
        assert_eq!(field(&response, "version"), string("HTTP/1.1"));
        assert_eq!(field(&response, "reason"), string("OK"));
        assert!(matches!(field(&response, "body"), Value::Object(handle) if matches!(&*handle.borrow(), Object::Array(items) if items.len() == 5)));

        let header_value = field(&response, "headers");
        assert!(matches!(header_value, Value::Object(handle) if matches!(&*handle.borrow(), Object::Dict(entries) if entries.iter().any(|(key, value)| key == &string("content-type") && value == &string("text/plain")))));

        server.join().expect("server thread");
    }

    #[test]
    fn http_request_parses_chunked_response_and_sends_body() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("listener");
        let address = listener.local_addr().expect("address");

        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut request = Vec::new();
            let mut buffer = [0u8; 1024];
            loop {
                let read = stream.read(&mut buffer).expect("request read");
                request.extend_from_slice(&buffer[..read]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            let body = b"abc";
            while request.len() < request.windows(4).position(|window| window == b"\r\n\r\n").unwrap() + 4 + body.len() {
                let read = stream.read(&mut buffer).expect("request body read");
                if read == 0 { break; }
                request.extend_from_slice(&buffer[..read]);
            }
            let request_text = String::from_utf8_lossy(&request);
            assert!(request_text.starts_with("POST /submit HTTP/1.1\r\n"));
            assert!(request_text.to_ascii_lowercase().contains("content-length: 3"));
            assert!(request.ends_with(body));

            stream
                .write_all(b"HTTP/1.1 201 Created\r\nTransfer-Encoding: chunked\r\nX-Test: chunked\r\n\r\n2\r\nhi\r\n3\r\n!ok\r\n0\r\n\r\n")
                .expect("chunked response");
        });

        let url = string(&format!("http://{}/submit", address));
        let response = native_http_request(&[
            string("POST"),
            url,
            headers(&[("X-Client", "kastel")]),
            string("abc"),
        ])
        .expect("http_request");

        assert_eq!(field(&response, "status"), Value::Integer(201));
        assert_eq!(field(&response, "reason"), string("Created"));
        assert!(matches!(field(&response, "body"), Value::Object(handle) if matches!(&*handle.borrow(), Object::Array(items) if items == &vec![int(b'h' as i64), int(b'i' as i64), int(b'!' as i64), int(b'o' as i64), int(b'k' as i64)])));

        server.join().expect("server thread");
    }
}
