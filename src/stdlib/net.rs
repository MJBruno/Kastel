//! Réseau standard de Kastel.
//!
//! La surface suit les primitives stables de `std::net` de Rust, avec une
//! adaptation au modèle d'exécution de Kastel : les sockets créées par cette
//! bibliothèque sont non bloquantes afin de ne pas immobiliser la VM
//! coopérative. Une lecture, réception ou acceptation sans données disponibles
//! retourne donc `None`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{
    IpAddr, Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr, SocketAddrV4, SocketAddrV6, TcpListener,
    TcpStream, ToSocketAddrs, UdpSocket,
};
use std::rc::Rc;
use std::time::Duration;

use crate::{
    error::runtime_error::RuntimeError,
    runtime::{net::NetworkState, object::Object, value::Value},
};

const MAX_NETWORK_BUFFER: usize = 16 * 1024 * 1024;
const MAX_ACCEPT_BATCH: usize = 1024;

fn expect_string(value: &Value) -> Result<String, RuntimeError> {
    value.as_string_value().ok_or(RuntimeError::TypeError)
}

fn expect_bool(value: &Value) -> Result<bool, RuntimeError> {
    match value {
        Value::Boolean(value) => Ok(*value),
        _ => Err(RuntimeError::TypeError),
    }
}

fn expect_i64(value: &Value) -> Result<i64, RuntimeError> {
    match value {
        Value::Integer(value) => Ok(*value),
        _ => Err(RuntimeError::TypeError),
    }
}

fn expect_port(value: &Value) -> Result<u16, RuntimeError> {
    match value {
        Value::Integer(port) if (0..=u16::MAX as i64).contains(port) => Ok(*port as u16),
        _ => Err(RuntimeError::TypeError),
    }
}

fn expect_u32(value: &Value) -> Result<u32, RuntimeError> {
    match value {
        Value::Integer(value) if (0..=u32::MAX as i64).contains(value) => Ok(*value as u32),
        _ => Err(RuntimeError::TypeError),
    }
}

fn expect_u16(value: &Value) -> Result<u16, RuntimeError> {
    match value {
        Value::Integer(value) if (0..=u16::MAX as i64).contains(value) => Ok(*value as u16),
        _ => Err(RuntimeError::TypeError),
    }
}

fn expect_u8(value: &Value) -> Result<u8, RuntimeError> {
    match value {
        Value::Integer(value) if (0..=u8::MAX as i64).contains(value) => Ok(*value as u8),
        _ => Err(RuntimeError::TypeError),
    }
}

fn expect_buffer_size(value: &Value) -> Result<usize, RuntimeError> {
    let Value::Integer(size) = value else {
        return Err(RuntimeError::TypeError);
    };

    let size = usize::try_from(*size).map_err(|_| RuntimeError::TypeError)?;
    if size > MAX_NETWORK_BUFFER {
        return Err(RuntimeError::ModuleError(format!(
            "net: requested buffer exceeds {} bytes",
            MAX_NETWORK_BUFFER
        )));
    }

    Ok(size)
}

fn expect_bytes(value: &Value) -> Result<Vec<u8>, RuntimeError> {
    let Value::Object(handle) = value else {
        return Err(RuntimeError::TypeError);
    };

    let object = handle.borrow();
    let elements = match &*object {
        Object::Array(elements) | Object::Tuple(elements) => elements,
        _ => return Err(RuntimeError::TypeError),
    };

    elements
        .iter()
        .map(|value| match value {
            Value::Integer(byte) if (0..=255).contains(byte) => Ok(*byte as u8),
            _ => Err(RuntimeError::TypeError),
        })
        .collect()
}

fn bytes_value(bytes: &[u8]) -> Value {
    Value::new_array(
        bytes
            .iter()
            .map(|byte| Value::Integer(*byte as i64))
            .collect(),
    )
}

fn ints_value(values: impl IntoIterator<Item = u32>) -> Value {
    Value::new_array(
        values
            .into_iter()
            .map(|value| Value::Integer(i64::from(value)))
            .collect(),
    )
}

fn io_error_kind_name(kind: std::io::ErrorKind) -> &'static str {
    match kind {
        std::io::ErrorKind::NotFound => "NotFound",
        std::io::ErrorKind::PermissionDenied => "PermissionDenied",
        std::io::ErrorKind::ConnectionRefused => "ConnectionRefused",
        std::io::ErrorKind::ConnectionReset => "ConnectionReset",
        std::io::ErrorKind::ConnectionAborted => "ConnectionAborted",
        std::io::ErrorKind::NotConnected => "NotConnected",
        std::io::ErrorKind::AddrInUse => "AddrInUse",
        std::io::ErrorKind::AddrNotAvailable => "AddrNotAvailable",
        std::io::ErrorKind::BrokenPipe => "BrokenPipe",
        std::io::ErrorKind::AlreadyExists => "AlreadyExists",
        std::io::ErrorKind::WouldBlock => "WouldBlock",
        std::io::ErrorKind::InvalidInput => "InvalidInput",
        std::io::ErrorKind::InvalidData => "InvalidData",
        std::io::ErrorKind::TimedOut => "TimedOut",
        std::io::ErrorKind::WriteZero => "WriteZero",
        std::io::ErrorKind::Interrupted => "Interrupted",
        std::io::ErrorKind::UnexpectedEof => "UnexpectedEof",
        std::io::ErrorKind::Unsupported => "Unsupported",
        std::io::ErrorKind::Other => "Other",
        _ => "Other",
    }
}

fn network_error(operation: &'static str, error: std::io::Error) -> RuntimeError {
    RuntimeError::NetworkError {
        operation,
        kind: io_error_kind_name(error.kind()),
        message: error.to_string(),
    }
}

fn network_method_operation(name: &str) -> &'static str {
    match name {
        "read" => "read",
        "peek" => "peek",
        "recv" => "recv",
        "recv_from" => "recv_from",
        "recv_from_addr" => "recv_from_addr",
        "set_multicast_loop_v4" => "set_multicast_loop_v4",
        "set_multicast_loop_v6" => "set_multicast_loop_v6",
        "multicast_loop_v4" => "multicast_loop_v4",
        "multicast_loop_v6" => "multicast_loop_v6",
        "join_multicast_v4" => "join_multicast_v4",
        "leave_multicast_v4" => "leave_multicast_v4",
        "join_multicast_v6" => "join_multicast_v6",
        "leave_multicast_v6" => "leave_multicast_v6",
        _ => "network_method",
    }
}

fn parse_error(operation: &'static str, message: impl Into<String>) -> RuntimeError {
    RuntimeError::NetworkError {
        operation,
        kind: "InvalidInput",
        message: message.into(),
    }
}

fn closed_error() -> RuntimeError {
    RuntimeError::NetworkError {
        operation: "socket",
        kind: "Closed",
        message: "socket is closed".into(),
    }
}

fn require_args(args: &[Value], expected: usize) -> Result<(), RuntimeError> {
    if args.len() != expected + 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected,
            found: args.len().saturating_sub(1),
        });
    }
    Ok(())
}

fn expect_network(value: &Value) -> Result<Rc<RefCell<NetworkState>>, RuntimeError> {
    let Value::Object(handle) = value else {
        return Err(RuntimeError::TypeError);
    };
    let object = handle.borrow();
    let Object::Network(network) = &*object else {
        return Err(RuntimeError::TypeError);
    };
    Ok(network.clone())
}

fn expect_ip(value: &Value) -> Result<IpAddr, RuntimeError> {
    let network = expect_network(value)?;
    let state = network.borrow();
    let NetworkState::IpAddr { address, .. } = &*state else {
        return Err(RuntimeError::TypeError);
    };
    Ok(*address)
}

fn expect_ipv4(value: &Value) -> Result<Ipv4Addr, RuntimeError> {
    let network = expect_network(value)?;
    let state = network.borrow();
    let NetworkState::Ipv4Addr { address, .. } = &*state else {
        return Err(RuntimeError::TypeError);
    };
    Ok(*address)
}

fn expect_ipv6(value: &Value) -> Result<Ipv6Addr, RuntimeError> {
    let network = expect_network(value)?;
    let state = network.borrow();
    let NetworkState::Ipv6Addr { address, .. } = &*state else {
        return Err(RuntimeError::TypeError);
    };
    Ok(*address)
}

fn expect_socket_addr(value: &Value) -> Result<SocketAddr, RuntimeError> {
    let network = expect_network(value)?;
    let state = network.borrow();
    let NetworkState::SocketAddr { address, .. } = &*state else {
        return Err(RuntimeError::TypeError);
    };
    Ok(*address)
}

fn optional_millis(value: &Value) -> Result<Option<Duration>, RuntimeError> {
    match value {
        Value::None => Ok(None),
        Value::Object(handle) => {
            let object = handle.borrow();
            match &*object {
                Object::Option(Some(Value::Integer(value))) if *value >= 0 => {
                    Ok(Some(Duration::from_millis(*value as u64)))
                }
                Object::Option(None) => Ok(None),
                _ => Err(RuntimeError::TypeError),
            }
        }
        _ => Err(RuntimeError::TypeError),
    }
}

fn millis_value(duration: Option<Duration>) -> Result<Value, RuntimeError> {
    match duration {
        None => Ok(Value::None),
        Some(duration) => {
            let millis = i64::try_from(duration.as_millis()).map_err(|_| {
                RuntimeError::ModuleError("net: timeout does not fit into int".into())
            })?;
            Ok(Value::new_some(Value::Integer(millis)))
        }
    }
}

fn error_option(error: Option<std::io::Error>) -> Value {
    match error {
        Some(error) => Value::new_some(Value::new_string(format!(
            "{}: {}",
            io_error_kind_name(error.kind()),
            error
        ))),
        None => Value::None,
    }
}

fn socket_addr_from_host_port(host: &str, port: u16) -> Result<SocketAddr, RuntimeError> {
    (host, port)
        .to_socket_addrs()
        .map_err(|error| network_error("resolve", error))?
        .next()
        .ok_or_else(|| parse_error("resolve", "host did not resolve to any address"))
}

fn network_value_for_socket(socket: TcpStream) -> Value {
    Value::new_network(NetworkState::tcp_stream(socket))
}

pub fn native_ip_parse(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let text = expect_string(&args[0])?;
    let address = text
        .parse::<IpAddr>()
        .map_err(|error| parse_error("ip_parse", error.to_string()))?;
    Ok(Value::new_network(NetworkState::ip_addr(address)))
}

pub fn native_ipv4_from_bits(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let bits = expect_u32(&args[0])?;
    Ok(Value::new_network(NetworkState::ipv4_addr(Ipv4Addr::from_bits(bits))))
}

pub fn native_ipv4_localhost(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }
    Ok(Value::new_network(NetworkState::ipv4_addr(Ipv4Addr::LOCALHOST)))
}

pub fn native_ipv4_unspecified(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }
    Ok(Value::new_network(NetworkState::ipv4_addr(Ipv4Addr::UNSPECIFIED)))
}

pub fn native_ipv4_broadcast(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }
    Ok(Value::new_network(NetworkState::ipv4_addr(Ipv4Addr::BROADCAST)))
}

pub fn native_ipv6_localhost(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }
    Ok(Value::new_network(NetworkState::ipv6_addr(Ipv6Addr::LOCALHOST)))
}

pub fn native_ipv6_unspecified(args: &[Value]) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 0,
            found: args.len(),
        });
    }
    Ok(Value::new_network(NetworkState::ipv6_addr(Ipv6Addr::UNSPECIFIED)))
}

pub fn native_ipv4(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 4 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 4,
            found: args.len(),
        });
    }

    let address = Ipv4Addr::new(
        expect_u8(&args[0])?,
        expect_u8(&args[1])?,
        expect_u8(&args[2])?,
        expect_u8(&args[3])?,
    );
    Ok(Value::new_network(NetworkState::ipv4_addr(address)))
}

pub fn native_ipv6(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 8 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 8,
            found: args.len(),
        });
    }

    let address = Ipv6Addr::new(
        expect_u16(&args[0])?,
        expect_u16(&args[1])?,
        expect_u16(&args[2])?,
        expect_u16(&args[3])?,
        expect_u16(&args[4])?,
        expect_u16(&args[5])?,
        expect_u16(&args[6])?,
        expect_u16(&args[7])?,
    );
    Ok(Value::new_network(NetworkState::ipv6_addr(address)))
}

pub fn native_socket_addr(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let address = SocketAddr::new(expect_ip(&args[0])?, expect_port(&args[1])?);
    Ok(Value::new_network(NetworkState::socket_addr(address)))
}

pub fn native_socket_addr_parse(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let text = expect_string(&args[0])?;
    let address = text
        .parse::<SocketAddr>()
        .map_err(|error| parse_error("socket_addr_parse", error.to_string()))?;
    Ok(Value::new_network(NetworkState::socket_addr(address)))
}

pub fn native_socket_addr_v4(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let address = SocketAddrV4::new(expect_ipv4(&args[0])?, expect_port(&args[1])?);
    Ok(Value::new_network(NetworkState::socket_addr_v4(address)))
}

pub fn native_socket_addr_v6(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 4 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 4,
            found: args.len(),
        });
    }

    let address = SocketAddrV6::new(
        expect_ipv6(&args[0])?,
        expect_port(&args[1])?,
        expect_u32(&args[2])?,
        expect_u32(&args[3])?,
    );
    Ok(Value::new_network(NetworkState::socket_addr_v6(address)))
}

pub fn native_resolve(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let host = expect_string(&args[0])?;
    let port = expect_port(&args[1])?;
    let addresses = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|error| network_error("resolve", error))?
        .map(|address| Value::new_network(NetworkState::socket_addr(address)))
        .collect();

    Ok(Value::new_array(addresses))
}

pub fn native_resolve_addr(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let value = expect_string(&args[0])?;
    let addresses = value
        .to_socket_addrs()
        .map_err(|error| network_error("resolve_addr", error))?
        .map(|address| Value::new_network(NetworkState::socket_addr(address)))
        .collect();

    Ok(Value::new_array(addresses))
}

pub fn native_tcp_connect(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let host = expect_string(&args[0])?;
    let port = expect_port(&args[1])?;
    let stream = TcpStream::connect((host.as_str(), port))
        .map_err(|error| network_error("tcp_connect", error))?;

    stream
        .set_nonblocking(true)
        .map_err(|error| network_error("tcp_connect", error))?;

    Ok(network_value_for_socket(stream))
}

pub fn native_tcp_connect_addr(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let address = expect_socket_addr(&args[0])?;
    let stream = TcpStream::connect(address)
        .map_err(|error| network_error("tcp_connect_addr", error))?;
    stream
        .set_nonblocking(true)
        .map_err(|error| network_error("tcp_connect_addr", error))?;
    Ok(network_value_for_socket(stream))
}

pub fn native_tcp_connect_timeout(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let address = expect_socket_addr(&args[0])?;
    let timeout_ms = expect_i64(&args[1])?;
    if timeout_ms < 0 {
        return Err(RuntimeError::TypeError);
    }

    let stream = TcpStream::connect_timeout(&address, Duration::from_millis(timeout_ms as u64))
        .map_err(|error| network_error("tcp_connect_timeout", error))?;
    stream
        .set_nonblocking(true)
        .map_err(|error| network_error("tcp_connect_timeout", error))?;
    Ok(network_value_for_socket(stream))
}

pub fn native_tcp_listen(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let host = expect_string(&args[0])?;
    let port = expect_port(&args[1])?;
    let listener = TcpListener::bind((host.as_str(), port))
        .map_err(|error| network_error("tcp_listen", error))?;

    listener
        .set_nonblocking(true)
        .map_err(|error| network_error("tcp_listen", error))?;

    Ok(Value::new_network(NetworkState::tcp_listener(listener)))
}

pub fn native_tcp_listen_addr(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let address = expect_socket_addr(&args[0])?;
    let listener = TcpListener::bind(address)
        .map_err(|error| network_error("tcp_listen_addr", error))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| network_error("tcp_listen_addr", error))?;
    Ok(Value::new_network(NetworkState::tcp_listener(listener)))
}

pub fn native_udp_bind(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 2,
            found: args.len(),
        });
    }

    let host = expect_string(&args[0])?;
    let port = expect_port(&args[1])?;
    let socket = UdpSocket::bind((host.as_str(), port))
        .map_err(|error| network_error("udp_bind", error))?;

    socket
        .set_nonblocking(true)
        .map_err(|error| network_error("udp_bind", error))?;

    Ok(Value::new_network(NetworkState::udp_socket(socket)))
}

pub fn native_udp_bind_addr(args: &[Value]) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected: 1,
            found: args.len(),
        });
    }

    let address = expect_socket_addr(&args[0])?;
    let socket = UdpSocket::bind(address)
        .map_err(|error| network_error("udp_bind_addr", error))?;
    socket
        .set_nonblocking(true)
        .map_err(|error| network_error("udp_bind_addr", error))?;
    Ok(Value::new_network(NetworkState::udp_socket(socket)))
}

fn tcp_stream_method(
    name: &str,
    args: &[Value],
    network: &Rc<RefCell<NetworkState>>,
) -> Result<Value, RuntimeError> {
    match name {
        "read" | "peek" => {
            require_args(args, 1)?;
            let size = expect_buffer_size(&args[1])?;
            let mut state = network.borrow_mut();
            let NetworkState::TcpStream { socket, .. } = &mut *state else {
                return Err(RuntimeError::TypeError);
            };
            let stream = socket.as_mut().ok_or_else(closed_error)?;
            if size == 0 {
                return Ok(Value::new_some(Value::new_array(Vec::new())));
            }

            let mut buffer = vec![0u8; size];
            let result = if name == "read" {
                stream.read(&mut buffer)
            } else {
                stream.peek(&mut buffer)
            };

            match result {
                Ok(0) => Ok(Value::new_some(Value::new_array(Vec::new()))),
                Ok(read) => Ok(Value::new_some(bytes_value(&buffer[..read]))),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(Value::None),
                Err(error) => Err(network_error(network_method_operation(name), error)),
            }
        }

        "write" => {
            require_args(args, 1)?;
            let bytes = expect_bytes(&args[1])?;
            let mut state = network.borrow_mut();
            let NetworkState::TcpStream { socket, .. } = &mut *state else {
                return Err(RuntimeError::TypeError);
            };
            let stream = socket.as_mut().ok_or_else(closed_error)?;

            match stream.write(&bytes) {
                Ok(written) => Ok(Value::Integer(written as i64)),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    Ok(Value::Integer(0))
                }
                Err(error) => Err(network_error("write", error)),
            }
        }

        "shutdown" => {
            require_args(args, 1)?;
            let mode = expect_string(&args[1])?;
            let shutdown = match mode.as_str() {
                "read" => Shutdown::Read,
                "write" => Shutdown::Write,
                "both" => Shutdown::Both,
                _ => return Err(RuntimeError::TypeError),
            };
            let state = network.borrow();
            let NetworkState::TcpStream { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let stream = socket.as_ref().ok_or_else(closed_error)?;
            stream
                .shutdown(shutdown)
                .map_err(|error| network_error("tcp_shutdown", error))?;
            Ok(Value::None)
        }

        "set_nodelay" => {
            require_args(args, 1)?;
            let enabled = expect_bool(&args[1])?;
            let state = network.borrow();
            let NetworkState::TcpStream { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let stream = socket.as_ref().ok_or_else(closed_error)?;
            stream
                .set_nodelay(enabled)
                .map_err(|error| network_error("set_nodelay", error))?;
            Ok(Value::None)
        }

        "nodelay" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::TcpStream { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let stream = socket.as_ref().ok_or_else(closed_error)?;
            Ok(Value::Boolean(
                stream
                    .nodelay()
                    .map_err(|error| network_error("nodelay", error))?,
            ))
        }

        "set_ttl" => {
            require_args(args, 1)?;
            let ttl = expect_u32(&args[1])?;
            let state = network.borrow();
            let NetworkState::TcpStream { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let stream = socket.as_ref().ok_or_else(closed_error)?;
            stream
                .set_ttl(ttl)
                .map_err(|error| network_error("set_ttl", error))?;
            Ok(Value::None)
        }

        "ttl" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::TcpStream { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let stream = socket.as_ref().ok_or_else(closed_error)?;
            Ok(Value::Integer(
                i64::from(stream.ttl().map_err(|error| network_error("ttl", error))?),
            ))
        }

        "set_nonblocking" => {
            require_args(args, 1)?;
            let nonblocking = expect_bool(&args[1])?;
            let mut state = network.borrow_mut();
            let NetworkState::TcpStream {
                socket,
                nonblocking: current,
                ..
            } = &mut *state
            else {
                return Err(RuntimeError::TypeError);
            };
            let stream = socket.as_ref().ok_or_else(closed_error)?;
            stream
                .set_nonblocking(nonblocking)
                .map_err(|error| network_error("set_nonblocking", error))?;
            current.set(nonblocking);
            Ok(Value::None)
        }

        "is_nonblocking" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::TcpStream { nonblocking, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            Ok(Value::Boolean(nonblocking.get()))
        }

        "set_read_timeout" | "set_write_timeout" => {
            require_args(args, 1)?;
            let duration = optional_millis(&args[1])?;
            let state = network.borrow();
            let NetworkState::TcpStream { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let stream = socket.as_ref().ok_or_else(closed_error)?;
            if name == "set_read_timeout" {
                stream
                    .set_read_timeout(duration)
                    .map_err(|error| network_error("set_read_timeout", error))?;
            } else {
                stream
                    .set_write_timeout(duration)
                    .map_err(|error| network_error("set_write_timeout", error))?;
            }
            Ok(Value::None)
        }

        "read_timeout" | "write_timeout" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::TcpStream { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let stream = socket.as_ref().ok_or_else(closed_error)?;
            if name == "read_timeout" {
                millis_value(
                    stream
                        .read_timeout()
                        .map_err(|error| network_error("read_timeout", error))?,
                )
            } else {
                millis_value(
                    stream
                        .write_timeout()
                        .map_err(|error| network_error("write_timeout", error))?,
                )
            }
        }

        "take_error" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::TcpStream { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let stream = socket.as_ref().ok_or_else(closed_error)?;
            Ok(error_option(
                stream
                    .take_error()
                    .map_err(|error| network_error("take_error", error))?,
            ))
        }

        "try_clone" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::TcpStream {
                socket,
                nonblocking,
                ..
            } = &*state
            else {
                return Err(RuntimeError::TypeError);
            };
            let stream = socket.as_ref().ok_or_else(closed_error)?;
            let cloned = stream
                .try_clone()
                .map_err(|error| network_error("try_clone", error))?;
            Ok(Value::new_network(NetworkState::tcp_stream_with_nonblocking(
                cloned,
                nonblocking.clone(),
            )))
        }

        "is_closed" => {
            require_args(args, 0)?;
            Ok(Value::Boolean(network.borrow().is_closed()))
        }

        "local_addr" | "local_socket_addr" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::TcpStream { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let stream = socket.as_ref().ok_or_else(closed_error)?;
            let address = stream
                .local_addr()
                .map_err(|error| network_error("local_addr", error))?;
            if name == "local_addr" {
                Ok(Value::new_string(address.to_string()))
            } else {
                Ok(Value::new_network(NetworkState::socket_addr(address)))
            }
        }

        "peer_addr" | "peer_socket_addr" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::TcpStream { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let stream = socket.as_ref().ok_or_else(closed_error)?;
            let address = stream
                .peer_addr()
                .map_err(|error| network_error("peer_addr", error))?;
            if name == "peer_addr" {
                Ok(Value::new_string(address.to_string()))
            } else {
                Ok(Value::new_network(NetworkState::socket_addr(address)))
            }
        }

        "close" => {
            require_args(args, 0)?;
            network.borrow_mut().close();
            Ok(Value::None)
        }

        _ => Err(RuntimeError::ObjectFieldNotFound {
            name: name.to_string(),
            suggestion: None,
        }),
    }
}

fn tcp_listener_method(
    name: &str,
    args: &[Value],
    network: &Rc<RefCell<NetworkState>>,
) -> Result<Value, RuntimeError> {
    match name {
        "accept" => {
            require_args(args, 0)?;
            let mut state = network.borrow_mut();
            let NetworkState::TcpListener { socket, .. } = &mut *state else {
                return Err(RuntimeError::TypeError);
            };
            let listener = socket.as_ref().ok_or_else(closed_error)?;

            match listener.accept() {
                Ok((stream, _address)) => {
                    stream
                        .set_nonblocking(true)
                        .map_err(|error| network_error("accept", error))?;
                    Ok(Value::new_some(Value::new_network(NetworkState::tcp_stream(
                        stream,
                    ))))
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(Value::None),
                Err(error) => Err(network_error("accept", error)),
            }
        }

        "accept_available" => {
            require_args(args, 1)?;
            let max = expect_i64(&args[1])?;
            if max < 0 || max as usize > MAX_ACCEPT_BATCH {
                return Err(RuntimeError::ModuleError(format!(
                    "net: accept batch must be between 0 and {}",
                    MAX_ACCEPT_BATCH
                )));
            }
            if max == 0 {
                return Ok(Value::new_array(Vec::new()));
            }

            let mut state = network.borrow_mut();
            let NetworkState::TcpListener { socket, .. } = &mut *state else {
                return Err(RuntimeError::TypeError);
            };
            let listener = socket.as_ref().ok_or_else(closed_error)?;
            let mut connections = Vec::new();

            for _ in 0..max {
                match listener.accept() {
                    Ok((stream, _address)) => {
                        stream
                            .set_nonblocking(true)
                            .map_err(|error| network_error("accept_available", error))?;
                        connections.push(Value::new_network(NetworkState::tcp_stream(stream)));
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(error) => return Err(network_error("accept_available", error)),
                }
            }

            Ok(Value::new_array(connections))
        }

        "try_clone" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::TcpListener {
                socket,
                nonblocking,
                ..
            } = &*state
            else {
                return Err(RuntimeError::TypeError);
            };
            let listener = socket.as_ref().ok_or_else(closed_error)?;
            let cloned = listener
                .try_clone()
                .map_err(|error| network_error("try_clone", error))?;
            Ok(Value::new_network(NetworkState::tcp_listener_with_nonblocking(
                cloned,
                nonblocking.clone(),
            )))
        }

        "set_ttl" => {
            require_args(args, 1)?;
            let ttl = expect_u32(&args[1])?;
            let state = network.borrow();
            let NetworkState::TcpListener { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let listener = socket.as_ref().ok_or_else(closed_error)?;
            listener
                .set_ttl(ttl)
                .map_err(|error| network_error("set_ttl", error))?;
            Ok(Value::None)
        }

        "ttl" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::TcpListener { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let listener = socket.as_ref().ok_or_else(closed_error)?;
            Ok(Value::Integer(
                i64::from(listener.ttl().map_err(|error| network_error("ttl", error))?),
            ))
        }

        "set_nonblocking" => {
            require_args(args, 1)?;
            let nonblocking = expect_bool(&args[1])?;
            let mut state = network.borrow_mut();
            let NetworkState::TcpListener {
                socket,
                nonblocking: current,
                ..
            } = &mut *state
            else {
                return Err(RuntimeError::TypeError);
            };
            let listener = socket.as_ref().ok_or_else(closed_error)?;
            listener
                .set_nonblocking(nonblocking)
                .map_err(|error| network_error("set_nonblocking", error))?;
            current.set(nonblocking);
            Ok(Value::None)
        }

        "is_nonblocking" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::TcpListener { nonblocking, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            Ok(Value::Boolean(nonblocking.get()))
        }

        "take_error" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::TcpListener { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let listener = socket.as_ref().ok_or_else(closed_error)?;
            Ok(error_option(
                listener
                    .take_error()
                    .map_err(|error| network_error("take_error", error))?,
            ))
        }

        "local_addr" | "local_socket_addr" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::TcpListener { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let listener = socket.as_ref().ok_or_else(closed_error)?;
            let address = listener
                .local_addr()
                .map_err(|error| network_error("local_addr", error))?;
            if name == "local_addr" {
                Ok(Value::new_string(address.to_string()))
            } else {
                Ok(Value::new_network(NetworkState::socket_addr(address)))
            }
        }

        "is_closed" => {
            require_args(args, 0)?;
            Ok(Value::Boolean(network.borrow().is_closed()))
        }

        "close" => {
            require_args(args, 0)?;
            network.borrow_mut().close();
            Ok(Value::None)
        }

        _ => Err(RuntimeError::ObjectFieldNotFound {
            name: name.to_string(),
            suggestion: None,
        }),
    }
}

fn udp_socket_method(
    name: &str,
    args: &[Value],
    network: &Rc<RefCell<NetworkState>>,
) -> Result<Value, RuntimeError> {
    match name {
        "send_to" => {
            require_args(args, 3)?;
            let bytes = expect_bytes(&args[1])?;
            let host = expect_string(&args[2])?;
            let port = expect_port(&args[3])?;
            let mut state = network.borrow_mut();
            let NetworkState::UdpSocket { socket, .. } = &mut *state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_mut().ok_or_else(closed_error)?;

            match socket.send_to(&bytes, (host.as_str(), port)) {
                Ok(written) => Ok(Value::Integer(written as i64)),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    Ok(Value::Integer(0))
                }
                Err(error) => Err(network_error("send_to", error)),
            }
        }

        "send_to_addr" => {
            require_args(args, 2)?;
            let bytes = expect_bytes(&args[1])?;
            let address = expect_socket_addr(&args[2])?;
            let mut state = network.borrow_mut();
            let NetworkState::UdpSocket { socket, .. } = &mut *state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_mut().ok_or_else(closed_error)?;
            match socket.send_to(&bytes, address) {
                Ok(written) => Ok(Value::Integer(written as i64)),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    Ok(Value::Integer(0))
                }
                Err(error) => Err(network_error("send_to_addr", error)),
            }
        }

        "recv_from" | "recv_from_addr" => {
            require_args(args, 1)?;
            let size = expect_buffer_size(&args[1])?;
            let mut state = network.borrow_mut();
            let NetworkState::UdpSocket { socket, .. } = &mut *state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_mut().ok_or_else(closed_error)?;
            let mut buffer = vec![0u8; size];

            match socket.recv_from(&mut buffer) {
                Ok((received, address)) => {
                    if name == "recv_from" {
                        Ok(Value::new_some(Value::new_tuple(vec![
                            bytes_value(&buffer[..received]),
                            Value::new_string(address.ip().to_string()),
                            Value::Integer(i64::from(address.port())),
                        ])))
                    } else {
                        Ok(Value::new_some(Value::new_tuple(vec![
                            bytes_value(&buffer[..received]),
                            Value::new_network(NetworkState::socket_addr(address)),
                        ])))
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(Value::None),
                Err(error) => Err(network_error(network_method_operation(name), error)),
            }
        }

        "peek_from" => {
            require_args(args, 1)?;
            let size = expect_buffer_size(&args[1])?;
            let mut state = network.borrow_mut();
            let NetworkState::UdpSocket { socket, .. } = &mut *state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_mut().ok_or_else(closed_error)?;
            let mut buffer = vec![0u8; size];
            match socket.peek_from(&mut buffer) {
                Ok((received, address)) => Ok(Value::new_some(Value::new_tuple(vec![
                    bytes_value(&buffer[..received]),
                    Value::new_network(NetworkState::socket_addr(address)),
                ]))),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(Value::None),
                Err(error) => Err(network_error("peek_from", error)),
            }
        }

        "connect" | "connect_addr" => {
            if name == "connect" {
                require_args(args, 2)?;
                let host = expect_string(&args[1])?;
                let port = expect_port(&args[2])?;
                let address = socket_addr_from_host_port(&host, port)?;
                let state = network.borrow();
                let NetworkState::UdpSocket { socket, .. } = &*state else {
                    return Err(RuntimeError::TypeError);
                };
                let socket = socket.as_ref().ok_or_else(closed_error)?;
                socket
                    .connect(address)
                    .map_err(|error| network_error("udp_connect", error))?;
            } else {
                require_args(args, 1)?;
                let address = expect_socket_addr(&args[1])?;
                let state = network.borrow();
                let NetworkState::UdpSocket { socket, .. } = &*state else {
                    return Err(RuntimeError::TypeError);
                };
                let socket = socket.as_ref().ok_or_else(closed_error)?;
                socket
                    .connect(address)
                    .map_err(|error| network_error("udp_connect_addr", error))?;
            }
            Ok(Value::None)
        }

        "send" | "recv" | "peek" => {
            if name == "send" {
                require_args(args, 1)?;
                let bytes = expect_bytes(&args[1])?;
                let state = network.borrow();
                let NetworkState::UdpSocket { socket, .. } = &*state else {
                    return Err(RuntimeError::TypeError);
                };
                let socket = socket.as_ref().ok_or_else(closed_error)?;
                match socket.send(&bytes) {
                    Ok(written) => Ok(Value::Integer(written as i64)),
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        Ok(Value::Integer(0))
                    }
                    Err(error) => Err(network_error("udp_send", error)),
                }
            } else {
                require_args(args, 1)?;
                let size = expect_buffer_size(&args[1])?;
                let state = network.borrow();
                let NetworkState::UdpSocket { socket, .. } = &*state else {
                    return Err(RuntimeError::TypeError);
                };
                let socket = socket.as_ref().ok_or_else(closed_error)?;
                let mut buffer = vec![0u8; size];
                let result = if name == "recv" {
                    socket.recv(&mut buffer)
                } else {
                    socket.peek(&mut buffer)
                };
                match result {
                    Ok(0) => Ok(Value::new_some(Value::new_array(Vec::new()))),
                    Ok(received) => Ok(Value::new_some(bytes_value(&buffer[..received]))),
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(Value::None),
                    Err(error) => Err(network_error(network_method_operation(name), error)),
                }
            }
        }

        "peer_addr" | "peer_socket_addr" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::UdpSocket { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            let address = socket
                .peer_addr()
                .map_err(|error| network_error("udp_peer_addr", error))?;
            if name == "peer_addr" {
                Ok(Value::new_string(address.to_string()))
            } else {
                Ok(Value::new_network(NetworkState::socket_addr(address)))
            }
        }

        "local_addr" | "local_socket_addr" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::UdpSocket { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            let address = socket
                .local_addr()
                .map_err(|error| network_error("local_addr", error))?;
            if name == "local_addr" {
                Ok(Value::new_string(address.to_string()))
            } else {
                Ok(Value::new_network(NetworkState::socket_addr(address)))
            }
        }

        "try_clone" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::UdpSocket {
                socket,
                nonblocking,
                ..
            } = &*state
            else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            let cloned = socket
                .try_clone()
                .map_err(|error| network_error("try_clone", error))?;
            Ok(Value::new_network(NetworkState::udp_socket_with_nonblocking(
                cloned,
                nonblocking.clone(),
            )))
        }

        "set_nonblocking" => {
            require_args(args, 1)?;
            let nonblocking = expect_bool(&args[1])?;
            let mut state = network.borrow_mut();
            let NetworkState::UdpSocket {
                socket,
                nonblocking: current,
                ..
            } = &mut *state
            else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            socket
                .set_nonblocking(nonblocking)
                .map_err(|error| network_error("set_nonblocking", error))?;
            current.set(nonblocking);
            Ok(Value::None)
        }

        "is_nonblocking" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::UdpSocket { nonblocking, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            Ok(Value::Boolean(nonblocking.get()))
        }

        "set_read_timeout" | "set_write_timeout" => {
            require_args(args, 1)?;
            let duration = optional_millis(&args[1])?;
            let state = network.borrow();
            let NetworkState::UdpSocket { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            if name == "set_read_timeout" {
                socket
                    .set_read_timeout(duration)
                    .map_err(|error| network_error("set_read_timeout", error))?;
            } else {
                socket
                    .set_write_timeout(duration)
                    .map_err(|error| network_error("set_write_timeout", error))?;
            }
            Ok(Value::None)
        }

        "read_timeout" | "write_timeout" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::UdpSocket { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            if name == "read_timeout" {
                millis_value(
                    socket
                        .read_timeout()
                        .map_err(|error| network_error("read_timeout", error))?,
                )
            } else {
                millis_value(
                    socket
                        .write_timeout()
                        .map_err(|error| network_error("write_timeout", error))?,
                )
            }
        }

        "take_error" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::UdpSocket { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            Ok(error_option(
                socket
                    .take_error()
                    .map_err(|error| network_error("take_error", error))?,
            ))
        }

        "set_broadcast" => {
            require_args(args, 1)?;
            let enabled = expect_bool(&args[1])?;
            let state = network.borrow();
            let NetworkState::UdpSocket { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            socket
                .set_broadcast(enabled)
                .map_err(|error| network_error("set_broadcast", error))?;
            Ok(Value::None)
        }

        "broadcast" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::UdpSocket { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            Ok(Value::Boolean(
                socket
                    .broadcast()
                    .map_err(|error| network_error("broadcast", error))?,
            ))
        }

        "set_ttl" => {
            require_args(args, 1)?;
            let ttl = expect_u32(&args[1])?;
            let state = network.borrow();
            let NetworkState::UdpSocket { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            socket
                .set_ttl(ttl)
                .map_err(|error| network_error("set_ttl", error))?;
            Ok(Value::None)
        }

        "ttl" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::UdpSocket { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            Ok(Value::Integer(
                i64::from(socket.ttl().map_err(|error| network_error("ttl", error))?),
            ))
        }

        "set_multicast_loop_v4" | "set_multicast_loop_v6" => {
            require_args(args, 1)?;
            let enabled = expect_bool(&args[1])?;
            let state = network.borrow();
            let NetworkState::UdpSocket { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            let result = if name.ends_with("_v4") {
                socket.set_multicast_loop_v4(enabled)
            } else {
                socket.set_multicast_loop_v6(enabled)
            };
            result.map_err(|error| network_error(network_method_operation(name), error))?;
            Ok(Value::None)
        }

        "multicast_loop_v4" | "multicast_loop_v6" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::UdpSocket { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            let value = if name.ends_with("_v4") {
                socket.multicast_loop_v4()
            } else {
                socket.multicast_loop_v6()
            }
            .map_err(|error| network_error(network_method_operation(name), error))?;
            Ok(Value::Boolean(value))
        }

        "set_multicast_ttl_v4" => {
            require_args(args, 1)?;
            let ttl = expect_u32(&args[1])?;
            let state = network.borrow();
            let NetworkState::UdpSocket { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            socket
                .set_multicast_ttl_v4(ttl)
                .map_err(|error| network_error("set_multicast_ttl_v4", error))?;
            Ok(Value::None)
        }

        "multicast_ttl_v4" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::UdpSocket { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            Ok(Value::Integer(i64::from(
                socket
                    .multicast_ttl_v4()
                    .map_err(|error| network_error("multicast_ttl_v4", error))?,
            )))
        }

        "join_multicast_v4" | "leave_multicast_v4" => {
            require_args(args, 2)?;
            let multi = expect_ipv4(&args[1])?;
            let interface = expect_ipv4(&args[2])?;
            let state = network.borrow();
            let NetworkState::UdpSocket { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            let result = if name.starts_with("join_") {
                socket.join_multicast_v4(&multi, &interface)
            } else {
                socket.leave_multicast_v4(&multi, &interface)
            };
            result.map_err(|error| network_error(network_method_operation(name), error))?;
            Ok(Value::None)
        }

        "join_multicast_v6" | "leave_multicast_v6" => {
            require_args(args, 2)?;
            let multi = expect_ipv6(&args[1])?;
            let interface = expect_u32(&args[2])?;
            let state = network.borrow();
            let NetworkState::UdpSocket { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            let result = if name.starts_with("join_") {
                socket.join_multicast_v6(&multi, interface)
            } else {
                socket.leave_multicast_v6(&multi, interface)
            };
            result.map_err(|error| network_error(network_method_operation(name), error))?;
            Ok(Value::None)
        }

        "is_closed" => {
            require_args(args, 0)?;
            Ok(Value::Boolean(network.borrow().is_closed()))
        }

        "close" => {
            require_args(args, 0)?;
            network.borrow_mut().close();
            Ok(Value::None)
        }

        _ => Err(RuntimeError::ObjectFieldNotFound {
            name: name.to_string(),
            suggestion: None,
        }),
    }
}

fn ip_addr_method(name: &str, args: &[Value], network: &Rc<RefCell<NetworkState>>) -> Result<Value, RuntimeError> {
    require_args(args, 0)?;
    let state = network.borrow();
    let NetworkState::IpAddr { address, .. } = &*state else {
        return Err(RuntimeError::TypeError);
    };

    match name {
        "is_v4" => Ok(Value::Boolean(address.is_ipv4())),
        "is_v6" => Ok(Value::Boolean(address.is_ipv6())),
        "is_unspecified" => Ok(Value::Boolean(address.is_unspecified())),
        "is_loopback" => Ok(Value::Boolean(address.is_loopback())),
        "is_multicast" => Ok(Value::Boolean(address.is_multicast())),
        "to_canonical" => Ok(Value::new_network(NetworkState::ip_addr(address.to_canonical()))),
        "as_ipv4" => match address {
            IpAddr::V4(value) => Ok(Value::new_some(Value::new_network(
                NetworkState::ipv4_addr(*value),
            ))),
            IpAddr::V6(_) => Ok(Value::None),
        },
        "as_ipv6" => match address {
            IpAddr::V4(_) => Ok(Value::None),
            IpAddr::V6(value) => Ok(Value::new_some(Value::new_network(
                NetworkState::ipv6_addr(*value),
            ))),
        },
        "to_string" => Ok(Value::new_string(address.to_string())),
        _ => Err(RuntimeError::ObjectFieldNotFound {
            name: name.to_string(),
            suggestion: None,
        }),
    }
}

fn ipv4_addr_method(name: &str, args: &[Value], network: &Rc<RefCell<NetworkState>>) -> Result<Value, RuntimeError> {
    require_args(args, 0)?;
    let state = network.borrow();
    let NetworkState::Ipv4Addr { address, .. } = &*state else {
        return Err(RuntimeError::TypeError);
    };

    match name {
        "octets" => Ok(bytes_value(&address.octets())),
        "to_bits" => Ok(Value::Integer(i64::from(address.to_bits()))),
        "is_broadcast" => Ok(Value::Boolean(address.is_broadcast())),
        "is_link_local" => Ok(Value::Boolean(address.is_link_local())),
        "is_loopback" => Ok(Value::Boolean(address.is_loopback())),
        "is_multicast" => Ok(Value::Boolean(address.is_multicast())),
        "is_private" => Ok(Value::Boolean(address.is_private())),
        "is_documentation" => Ok(Value::Boolean(address.is_documentation())),
        "is_unspecified" => Ok(Value::Boolean(address.is_unspecified())),
        "to_ipv6_compatible" => Ok(Value::new_network(NetworkState::ipv6_addr(
            address.to_ipv6_compatible(),
        ))),
        "to_ipv6_mapped" => Ok(Value::new_network(NetworkState::ipv6_addr(
            address.to_ipv6_mapped(),
        ))),
        "to_ip" => Ok(Value::new_network(NetworkState::ip_addr(IpAddr::V4(*address)))),
        "to_string" => Ok(Value::new_string(address.to_string())),
        _ => Err(RuntimeError::ObjectFieldNotFound {
            name: name.to_string(),
            suggestion: None,
        }),
    }
}

fn ipv6_addr_method(name: &str, args: &[Value], network: &Rc<RefCell<NetworkState>>) -> Result<Value, RuntimeError> {
    require_args(args, 0)?;
    let state = network.borrow();
    let NetworkState::Ipv6Addr { address, .. } = &*state else {
        return Err(RuntimeError::TypeError);
    };

    match name {
        "segments" => Ok(ints_value(address.segments().into_iter().map(u32::from))),
        "octets" => Ok(bytes_value(&address.octets())),
        "is_loopback" => Ok(Value::Boolean(address.is_loopback())),
        "is_multicast" => Ok(Value::Boolean(address.is_multicast())),
        "is_unique_local" => Ok(Value::Boolean(address.is_unique_local())),
        "is_unicast_link_local" => Ok(Value::Boolean(address.is_unicast_link_local())),
        "is_unspecified" => Ok(Value::Boolean(address.is_unspecified())),
        "is_ipv4_mapped" => Ok(Value::Boolean(address.to_ipv4().is_some())),
        "to_ipv4" => match address.to_ipv4() {
            Some(value) => Ok(Value::new_some(Value::new_network(NetworkState::ipv4_addr(value)))),
            None => Ok(Value::None),
        },
        "to_ip" | "to_canonical" => Ok(Value::new_network(NetworkState::ip_addr(IpAddr::V6(
            *address,
        )))),
        "to_string" => Ok(Value::new_string(address.to_string())),
        _ => Err(RuntimeError::ObjectFieldNotFound {
            name: name.to_string(),
            suggestion: None,
        }),
    }
}

fn socket_addr_method(name: &str, args: &[Value], network: &Rc<RefCell<NetworkState>>) -> Result<Value, RuntimeError> {
    match name {
        "ip" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::SocketAddr { address, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            Ok(Value::new_network(NetworkState::ip_addr(address.ip())))
        }
        "port" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::SocketAddr { address, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            Ok(Value::Integer(i64::from(address.port())))
        }
        "is_ipv4" | "is_ipv6" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::SocketAddr { address, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            Ok(Value::Boolean(if name == "is_ipv4" {
                address.is_ipv4()
            } else {
                address.is_ipv6()
            }))
        }
        "as_v4" | "as_v6" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::SocketAddr { address, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            match (name, address) {
                ("as_v4", SocketAddr::V4(value)) => Ok(Value::new_some(Value::new_network(
                    NetworkState::socket_addr_v4(*value),
                ))),
                ("as_v6", SocketAddr::V6(value)) => Ok(Value::new_some(Value::new_network(
                    NetworkState::socket_addr_v6(*value),
                ))),
                _ => Ok(Value::None),
            }
        }
        "set_ip" => {
            require_args(args, 1)?;
            let ip = expect_ip(&args[1])?;
            let mut state = network.borrow_mut();
            let NetworkState::SocketAddr { address, .. } = &mut *state else {
                return Err(RuntimeError::TypeError);
            };
            address.set_ip(ip);
            Ok(Value::None)
        }
        "set_port" => {
            require_args(args, 1)?;
            let port = expect_port(&args[1])?;
            let mut state = network.borrow_mut();
            let NetworkState::SocketAddr { address, .. } = &mut *state else {
                return Err(RuntimeError::TypeError);
            };
            address.set_port(port);
            Ok(Value::None)
        }
        "to_string" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::SocketAddr { address, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            Ok(Value::new_string(address.to_string()))
        }
        _ => Err(RuntimeError::ObjectFieldNotFound {
            name: name.to_string(),
            suggestion: None,
        }),
    }
}

fn socket_addr_v4_method(name: &str, args: &[Value], network: &Rc<RefCell<NetworkState>>) -> Result<Value, RuntimeError> {
    match name {
        "ip" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::SocketAddrV4 { address, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            Ok(Value::new_network(NetworkState::ipv4_addr(*address.ip())))
        }
        "port" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::SocketAddrV4 { address, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            Ok(Value::Integer(i64::from(address.port())))
        }
        "set_ip" => {
            require_args(args, 1)?;
            let ip = expect_ipv4(&args[1])?;
            let mut state = network.borrow_mut();
            let NetworkState::SocketAddrV4 { address, .. } = &mut *state else {
                return Err(RuntimeError::TypeError);
            };
            address.set_ip(ip);
            Ok(Value::None)
        }
        "set_port" => {
            require_args(args, 1)?;
            let port = expect_port(&args[1])?;
            let mut state = network.borrow_mut();
            let NetworkState::SocketAddrV4 { address, .. } = &mut *state else {
                return Err(RuntimeError::TypeError);
            };
            address.set_port(port);
            Ok(Value::None)
        }
        "to_socket_addr" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::SocketAddrV4 { address, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            Ok(Value::new_network(NetworkState::socket_addr((*address).into())))
        }
        "to_string" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::SocketAddrV4 { address, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            Ok(Value::new_string(address.to_string()))
        }
        _ => Err(RuntimeError::ObjectFieldNotFound {
            name: name.to_string(),
            suggestion: None,
        }),
    }
}

fn socket_addr_v6_method(name: &str, args: &[Value], network: &Rc<RefCell<NetworkState>>) -> Result<Value, RuntimeError> {
    match name {
        "ip" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::SocketAddrV6 { address, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            Ok(Value::new_network(NetworkState::ipv6_addr(*address.ip())))
        }
        "port" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::SocketAddrV6 { address, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            Ok(Value::Integer(i64::from(address.port())))
        }
        "flowinfo" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::SocketAddrV6 { address, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            Ok(Value::Integer(i64::from(address.flowinfo())))
        }
        "scope_id" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::SocketAddrV6 { address, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            Ok(Value::Integer(i64::from(address.scope_id())))
        }
        "set_ip" => {
            require_args(args, 1)?;
            let ip = expect_ipv6(&args[1])?;
            let mut state = network.borrow_mut();
            let NetworkState::SocketAddrV6 { address, .. } = &mut *state else {
                return Err(RuntimeError::TypeError);
            };
            address.set_ip(ip);
            Ok(Value::None)
        }
        "set_port" => {
            require_args(args, 1)?;
            let port = expect_port(&args[1])?;
            let mut state = network.borrow_mut();
            let NetworkState::SocketAddrV6 { address, .. } = &mut *state else {
                return Err(RuntimeError::TypeError);
            };
            address.set_port(port);
            Ok(Value::None)
        }
        "set_flowinfo" => {
            require_args(args, 1)?;
            let flowinfo = expect_u32(&args[1])?;
            let mut state = network.borrow_mut();
            let NetworkState::SocketAddrV6 { address, .. } = &mut *state else {
                return Err(RuntimeError::TypeError);
            };
            address.set_flowinfo(flowinfo);
            Ok(Value::None)
        }
        "set_scope_id" => {
            require_args(args, 1)?;
            let scope_id = expect_u32(&args[1])?;
            let mut state = network.borrow_mut();
            let NetworkState::SocketAddrV6 { address, .. } = &mut *state else {
                return Err(RuntimeError::TypeError);
            };
            address.set_scope_id(scope_id);
            Ok(Value::None)
        }
        "to_socket_addr" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::SocketAddrV6 { address, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            Ok(Value::new_network(NetworkState::socket_addr((*address).into())))
        }
        "to_string" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::SocketAddrV6 { address, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            Ok(Value::new_string(address.to_string()))
        }
        _ => Err(RuntimeError::ObjectFieldNotFound {
            name: name.to_string(),
            suggestion: None,
        }),
    }
}

/// Dispatch des méthodes réseau spécialisées du VM.
pub fn dispatch_method(name: &str, args: &[Value]) -> Result<Option<Value>, RuntimeError> {
    let Some(Value::Object(handle)) = args.first() else {
        return Ok(None);
    };

    let network = {
        let object = handle.borrow();
        let Object::Network(network) = &*object else {
            return Ok(None);
        };
        network.clone()
    };

    let kind = network.borrow().type_name();
    let result = match kind {
        "TcpStream" => tcp_stream_method(name, args, &network),
        "TcpListener" => tcp_listener_method(name, args, &network),
        "UdpSocket" => udp_socket_method(name, args, &network),
        "IpAddr" => ip_addr_method(name, args, &network),
        "Ipv4Addr" => ipv4_addr_method(name, args, &network),
        "Ipv6Addr" => ipv6_addr_method(name, args, &network),
        "SocketAddr" => socket_addr_method(name, args, &network),
        "SocketAddrV4" => socket_addr_v4_method(name, args, &network),
        "SocketAddrV6" => socket_addr_v6_method(name, args, &network),
        _ => return Ok(None),
    }?;

    Ok(Some(result))
}

pub fn register(globals: &mut HashMap<String, Value>) {
    globals.insert("ip_parse".into(), Value::NativeFunction(native_ip_parse));
    globals.insert("ipv4".into(), Value::NativeFunction(native_ipv4));
    globals.insert("ipv4_from_bits".into(), Value::NativeFunction(native_ipv4_from_bits));
    globals.insert("ipv4_localhost".into(), Value::NativeFunction(native_ipv4_localhost));
    globals.insert("ipv4_unspecified".into(), Value::NativeFunction(native_ipv4_unspecified));
    globals.insert("ipv4_broadcast".into(), Value::NativeFunction(native_ipv4_broadcast));
    globals.insert("ipv6_localhost".into(), Value::NativeFunction(native_ipv6_localhost));
    globals.insert("ipv6_unspecified".into(), Value::NativeFunction(native_ipv6_unspecified));
    globals.insert("ipv6".into(), Value::NativeFunction(native_ipv6));
    globals.insert(
        "socket_addr".into(),
        Value::NativeFunction(native_socket_addr),
    );
    globals.insert(
        "socket_addr_parse".into(),
        Value::NativeFunction(native_socket_addr_parse),
    );
    globals.insert(
        "socket_addr_v4".into(),
        Value::NativeFunction(native_socket_addr_v4),
    );
    globals.insert(
        "socket_addr_v6".into(),
        Value::NativeFunction(native_socket_addr_v6),
    );
    globals.insert("resolve".into(), Value::NativeFunction(native_resolve));
    globals.insert(
        "resolve_addr".into(),
        Value::NativeFunction(native_resolve_addr),
    );
    globals.insert(
        "tcp_connect".into(),
        Value::NativeFunction(native_tcp_connect),
    );
    globals.insert(
        "tcp_connect_addr".into(),
        Value::NativeFunction(native_tcp_connect_addr),
    );
    globals.insert(
        "tcp_connect_timeout".into(),
        Value::NativeFunction(native_tcp_connect_timeout),
    );
    globals.insert(
        "tcp_listen".into(),
        Value::NativeFunction(native_tcp_listen),
    );
    globals.insert(
        "tcp_listen_addr".into(),
        Value::NativeFunction(native_tcp_listen_addr),
    );
    globals.insert("udp_bind".into(), Value::NativeFunction(native_udp_bind));
    globals.insert(
        "udp_bind_addr".into(),
        Value::NativeFunction(native_udp_bind_addr),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::object::Object;

    fn int(value: i64) -> Value {
        Value::Integer(value)
    }

    fn string(value: &str) -> Value {
        Value::new_string(value.to_string())
    }

    fn bytes(values: &[u8]) -> Value {
        bytes_value(values)
    }

    #[test]
    fn address_values_have_structural_equality_and_hashing() {
        let left = native_ipv4(&[int(192), int(168), int(1), int(10)]).unwrap();
        let right = native_ipv4(&[int(192), int(168), int(1), int(10)]).unwrap();

        assert_eq!(left, right);
        assert_eq!(left.key_hash(), right.key_hash());

        let dict = Value::new_dict(vec![(left, int(42))]);
        assert_eq!(dict.dict_get(&right).unwrap(), int(42));
    }

    #[test]
    fn address_values_round_trip_and_classify() {
        let ipv4 = native_ipv4(&[int(127), int(0), int(0), int(1)]).unwrap();
        let ip = ipv4_addr_method(
            "to_ip",
            &[ipv4.clone()],
            &expect_network(&ipv4).unwrap(),
        )
        .unwrap();
        let canonical = ip_addr_method("to_canonical", &[ip.clone()], &expect_network(&ip).unwrap()).unwrap();

        assert_eq!(canonical.to_string(), "127.0.0.1");
        assert_eq!(ip_addr_method("is_v4", &[ip.clone()], &expect_network(&ip).unwrap()).unwrap(), Value::Boolean(true));
        assert_eq!(ipv4_addr_method("is_loopback", &[ipv4.clone()], &expect_network(&ipv4).unwrap()).unwrap(), Value::Boolean(true));

        let parsed = native_socket_addr_parse(&[string("127.0.0.1:8080")]).unwrap();
        assert_eq!(socket_addr_method("port", &[parsed.clone()], &expect_network(&parsed).unwrap()).unwrap(), int(8080));
        assert_eq!(parsed.to_string(), "127.0.0.1:8080");
    }

    #[test]
    fn constructors_and_resolution_work_on_loopback() {
        let parsed = native_ip_parse(&[string("::1")]).unwrap();
        assert_eq!(parsed.to_string(), "::1");

        let addresses = native_resolve(&[string("127.0.0.1"), int(80)]).unwrap();
        let Value::Object(handle) = addresses else {
            panic!("resolve doit retourner une liste");
        };
        let object = handle.borrow();
        assert!(matches!(&*object, Object::Array(values) if !values.is_empty()));

        let resolved = native_resolve_addr(&[string("127.0.0.1:80")]).unwrap();
        let Value::Object(handle) = resolved else {
            panic!("resolve_addr doit retourner une liste");
        };
        let object = handle.borrow();
        assert!(matches!(&*object, Object::Array(values) if !values.is_empty()));
    }

    #[test]
    fn tcp_connect_and_listener_use_typed_socket_addresses() {
        let listener = native_tcp_listen(&[string("127.0.0.1"), int(0)]).unwrap();
        let address = dispatch_method("local_socket_addr", std::slice::from_ref(&listener))
            .unwrap()
            .unwrap();
        let client = native_tcp_connect_addr(std::slice::from_ref(&address)).unwrap();

        assert_eq!(
            dispatch_method("is_nonblocking", std::slice::from_ref(&client))
                .unwrap()
                .unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            dispatch_method("peer_socket_addr", std::slice::from_ref(&client))
                .unwrap()
                .unwrap()
                .to_string()
                .parse::<SocketAddr>()
                .is_ok(),
            true
        );

        let accepted = loop {
            let value = dispatch_method("accept", std::slice::from_ref(&listener))
                .unwrap()
                .unwrap();
            if matches!(value, Value::None) {
                std::thread::sleep(Duration::from_millis(1));
                continue;
            }
            let Value::Object(option_handle) = value else {
                panic!("accept doit retourner Option<TcpStream>");
            };
            let object = option_handle.borrow();
            let Object::Option(Some(Value::Object(stream_handle))) = &*object else {
                panic!("accept doit retourner Some(TcpStream)");
            };
            break Value::Object(stream_handle.clone());
        };

        assert_eq!(
            dispatch_method("nodelay", std::slice::from_ref(&client))
                .unwrap()
                .unwrap(),
            Value::Boolean(false)
        );
        dispatch_method("set_nodelay", &[client.clone(), Value::Boolean(true)])
            .unwrap();
        assert_eq!(
            dispatch_method("nodelay", std::slice::from_ref(&client))
                .unwrap()
                .unwrap(),
            Value::Boolean(true)
        );

        dispatch_method("close", std::slice::from_ref(&client)).unwrap();
        dispatch_method("close", std::slice::from_ref(&accepted)).unwrap();
        dispatch_method("close", std::slice::from_ref(&listener)).unwrap();
    }

    #[test]
    fn tcp_peek_timeout_clone_and_accept_batch_are_available() {
        let listener = native_tcp_listen(&[string("127.0.0.1"), int(0)]).unwrap();
        let address = dispatch_method("local_socket_addr", std::slice::from_ref(&listener))
            .unwrap()
            .unwrap();
        let client = native_tcp_connect_addr(std::slice::from_ref(&address)).unwrap();
        let clone = dispatch_method("try_clone", std::slice::from_ref(&client))
            .unwrap()
            .unwrap();

        dispatch_method(
            "set_read_timeout",
            &[client.clone(), Value::new_some(int(100))],
        )
        .unwrap();
        assert_eq!(
            dispatch_method("read_timeout", std::slice::from_ref(&client))
                .unwrap()
                .unwrap()
                .to_string(),
            "Some(100)"
        );

        let batch = dispatch_method("accept_available", &[listener.clone(), int(4)])
            .unwrap()
            .unwrap();
        let Value::Object(batch_handle) = batch else {
            panic!("accept_available doit retourner une liste");
        };
        assert!(matches!(&*batch_handle.borrow(), Object::Array(values) if !values.is_empty()));

        let payload = bytes(&[10, 20, 30]);
        let written = dispatch_method("write", &[client.clone(), payload]).unwrap().unwrap();
        assert!(matches!(written, Value::Integer(count) if count > 0));

        let _peek = dispatch_method("peek", &[clone.clone(), int(3)]).unwrap().unwrap();

        dispatch_method("close", std::slice::from_ref(&client)).unwrap();
        dispatch_method("close", std::slice::from_ref(&clone)).unwrap();
        dispatch_method("close", std::slice::from_ref(&listener)).unwrap();
    }

    #[test]
    fn udp_extended_surface_works_on_loopback() {
        let sender = native_udp_bind(&[string("127.0.0.1"), int(0)]).unwrap();
        let receiver = native_udp_bind(&[string("127.0.0.1"), int(0)]).unwrap();
        let address = dispatch_method("local_socket_addr", std::slice::from_ref(&receiver))
            .unwrap()
            .unwrap();

        assert_eq!(
            dispatch_method("broadcast", std::slice::from_ref(&sender))
                .unwrap()
                .unwrap(),
            Value::Boolean(false)
        );
        dispatch_method("set_broadcast", &[sender.clone(), Value::Boolean(true)]).unwrap();
        assert_eq!(
            dispatch_method("broadcast", std::slice::from_ref(&sender))
                .unwrap()
                .unwrap(),
            Value::Boolean(true)
        );

        dispatch_method("send_to_addr", &[sender.clone(), bytes(&[1, 2, 3]), address.clone()])
            .unwrap();

        let mut packet = Value::None;
        for _ in 0..200 {
            packet = dispatch_method("recv_from_addr", &[receiver.clone(), int(16)])
                .unwrap()
                .unwrap();
            if !matches!(packet, Value::None) {
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(matches!(packet, Value::Object(handle) if matches!(&*handle.borrow(), Object::Option(Some(Value::Object(tuple))) if matches!(&*tuple.borrow(), Object::Tuple(items) if items.len() == 2))));

        assert_eq!(
            dispatch_method("try_clone", std::slice::from_ref(&sender))
                .unwrap()
                .unwrap()
                .to_string(),
            "UdpSocket"
        );

        dispatch_method("close", std::slice::from_ref(&sender)).unwrap();
        dispatch_method("close", std::slice::from_ref(&receiver)).unwrap();
    }

    #[test]
    fn rejects_invalid_addresses_and_arguments() {
        assert!(matches!(
            native_ip_parse(&[string("not-an-ip")]),
            Err(RuntimeError::NetworkError { kind: "InvalidInput", .. })
        ));
        assert!(matches!(
            native_tcp_connect(&[string("127.0.0.1"), int(-1)]),
            Err(RuntimeError::TypeError)
        ));
        assert!(matches!(
            native_tcp_listen_addr(&[string("127.0.0.1:1")]),
            Err(RuntimeError::TypeError)
        ));
    }
}
