//! Réseau standard de Kastel.
//!
//! La première couche reste volontairement petite et portable :
//! - TCP client : `tcp_connect(host, port)`
//! - TCP serveur : `tcp_listen(host, port)` puis `listener.accept()`
//! - UDP : `udp_bind(host, port)` puis `send_to` / `recv_from`
//!
//! Les sockets sont non bloquants afin qu'une opération réseau n'immobilise
//! pas toute la VM coopérative. Les opérations de lecture/réception et
//! `accept()` retournent donc `None` lorsqu'aucune donnée n'est disponible.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream, UdpSocket};
use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    error::runtime_error::RuntimeError,
    runtime::{net::NetworkState, object::Object, value::Value},
};

const MAX_NETWORK_BUFFER: usize = 16 * 1024 * 1024;

fn expect_string(value: &Value) -> Result<String, RuntimeError> {
    value.as_string_value().ok_or(RuntimeError::TypeError)
}

fn expect_port(value: &Value) -> Result<u16, RuntimeError> {
    match value {
        Value::Integer(port) if (0..=u16::MAX as i64).contains(port) => Ok(*port as u16),
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

fn closed_error() -> RuntimeError {
    RuntimeError::NetworkError {
        operation: "socket",
        kind: "Closed",
        message: "socket is closed".into(),
    }
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

    Ok(Value::new_network(NetworkState::tcp_stream(stream)))
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

fn require_args(args: &[Value], expected: usize) -> Result<(), RuntimeError> {
    if args.len() != expected + 1 {
        return Err(RuntimeError::WrongArgumentCount {
            expected,
            found: args.len().saturating_sub(1),
        });
    }
    Ok(())
}

fn tcp_stream_method(name: &str, args: &[Value], network: &Rc<RefCell<NetworkState>>) -> Result<Value, RuntimeError> {
    match name {
        "read" => {
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
            match stream.read(&mut buffer) {
                Ok(0) => Ok(Value::new_some(Value::new_array(Vec::new()))),
                Ok(read) => Ok(Value::new_some(bytes_value(&buffer[..read]))),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(Value::None),
                Err(error) => Err(network_error("read", error)),
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
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(Value::Integer(0)),
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
            let mut state = network.borrow_mut();
            let NetworkState::TcpStream { socket, .. } = &mut *state else {
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
            let enabled = match args[1] {
                Value::Boolean(value) => value,
                _ => return Err(RuntimeError::TypeError),
            };
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

        "is_closed" => {
            require_args(args, 0)?;
            Ok(Value::Boolean(network.borrow().is_closed()))
        }

        "local_addr" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::TcpStream { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let stream = socket.as_ref().ok_or_else(closed_error)?;
            let address = stream
                .local_addr()
                .map_err(|error| network_error("local_addr", error))?;
            Ok(Value::new_string(address.to_string()))
        }

        "peer_addr" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::TcpStream { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let stream = socket.as_ref().ok_or_else(closed_error)?;
            let address = stream
                .peer_addr()
                .map_err(|error| network_error("peer_addr", error))?;
            Ok(Value::new_string(address.to_string()))
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
                    Ok(Value::new_some(Value::new_network(NetworkState::tcp_stream(stream))))
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(Value::None),
                Err(error) => Err(network_error("accept", error)),
            }
        }

        "local_addr" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::TcpListener { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let listener = socket.as_ref().ok_or_else(closed_error)?;
            let address = listener
                .local_addr()
                .map_err(|error| network_error("local_addr", error))?;
            Ok(Value::new_string(address.to_string()))
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

fn udp_socket_method(name: &str, args: &[Value], network: &Rc<RefCell<NetworkState>>) -> Result<Value, RuntimeError> {
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
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(Value::Integer(0)),
                Err(error) => Err(network_error("send_to", error)),
            }
        }

        "recv_from" => {
            require_args(args, 1)?;
            let size = expect_buffer_size(&args[1])?;
            let mut state = network.borrow_mut();
            let NetworkState::UdpSocket { socket, .. } = &mut *state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_mut().ok_or_else(closed_error)?;
            let mut buffer = vec![0u8; size];

            match socket.recv_from(&mut buffer) {
                Ok((received, address)) => Ok(Value::new_some(Value::new_tuple(vec![
                    bytes_value(&buffer[..received]),
                    Value::new_string(address.ip().to_string()),
                    Value::Integer(i64::from(address.port())),
                ]))),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(Value::None),
                Err(error) => Err(network_error("recv_from", error)),
            }
        }

        "connect" => {
            require_args(args, 2)?;
            let host = expect_string(&args[1])?;
            let port = expect_port(&args[2])?;
            let mut state = network.borrow_mut();
            let NetworkState::UdpSocket { socket, .. } = &mut *state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            socket
                .connect((host.as_str(), port))
                .map_err(|error| network_error("udp_connect", error))?;
            Ok(Value::None)
        }

        "send" => {
            require_args(args, 1)?;
            let bytes = expect_bytes(&args[1])?;
            let mut state = network.borrow_mut();
            let NetworkState::UdpSocket { socket, .. } = &mut *state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            match socket.send(&bytes) {
                Ok(written) => Ok(Value::Integer(written as i64)),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(Value::Integer(0)),
                Err(error) => Err(network_error("udp_send", error)),
            }
        }

        "peer_addr" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::UdpSocket { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            let address = socket
                .peer_addr()
                .map_err(|error| network_error("udp_peer_addr", error))?;
            Ok(Value::new_string(address.to_string()))
        }

        "local_addr" => {
            require_args(args, 0)?;
            let state = network.borrow();
            let NetworkState::UdpSocket { socket, .. } = &*state else {
                return Err(RuntimeError::TypeError);
            };
            let socket = socket.as_ref().ok_or_else(closed_error)?;
            let address = socket
                .local_addr()
                .map_err(|error| network_error("local_addr", error))?;
            Ok(Value::new_string(address.to_string()))
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
        _ => return Ok(None),
    }?;

    Ok(Some(result))
}

fn register_one(globals: &mut HashMap<String, Value>, name: &str, function: super::NativeFn) {
    globals.insert(name.to_string(), Value::NativeFunction(function));
}

pub fn register(globals: &mut HashMap<String, Value>) {
    register_one(globals, "tcp_connect", native_tcp_connect);
    register_one(globals, "tcp_listen", native_tcp_listen);
    register_one(globals, "udp_bind", native_udp_bind);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn string(value: &str) -> Value {
        Value::new_string(value.to_string())
    }

    fn int(value: i64) -> Value {
        Value::Integer(value)
    }

    fn bytes(values: &[i64]) -> Value {
        Value::new_array(values.iter().copied().map(Value::Integer).collect())
    }

    fn parse_socket_address(value: &Value) -> (String, i64) {
        let address = value.as_string_value().expect("adresse réseau en str");
        let parsed = address.parse::<std::net::SocketAddr>().expect("SocketAddr valide");
        (parsed.ip().to_string(), i64::from(parsed.port()))
    }

    #[test]
    fn constructors_accept_loopback_and_ephemeral_ports() {
        let listener = native_tcp_listen(&[string("127.0.0.1"), int(0)]).unwrap();
        let udp = native_udp_bind(&[string("127.0.0.1"), int(0)]).unwrap();

        assert!(matches!(listener.type_name(), "TcpListener"));
        assert!(matches!(udp.type_name(), "UdpSocket"));
    }

    #[test]
    fn tcp_accept_write_and_read_work_without_blocking() {
        let listener = native_tcp_listen(&[string("127.0.0.1"), int(0)]).unwrap();
        let address = dispatch_method("local_addr", std::slice::from_ref(&listener))
            .unwrap()
            .unwrap();
        let (host, port) = parse_socket_address(&address);

        let before = dispatch_method("accept", std::slice::from_ref(&listener))
            .unwrap()
            .unwrap();
        assert_eq!(before, Value::None);

        let client = native_tcp_connect(&[string(&host), int(port)]).unwrap();
        let accepted = loop {
            let value = dispatch_method("accept", std::slice::from_ref(&listener))
                .unwrap()
                .unwrap();
            if matches!(value, Value::None) {
                std::thread::sleep(std::time::Duration::from_millis(1));
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

        let payload = bytes(&[72, 105]);
        let mut total_written = 0i64;
        for _ in 0..200 {
            let written = dispatch_method("write", &[client.clone(), payload.clone()])
                .unwrap()
                .unwrap();
            let Value::Integer(count) = written else {
                panic!("write doit renvoyer un entier");
            };
            total_written = total_written.max(count);
            if count > 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(total_written > 0, "le socket TCP doit accepter au moins un octet");

        let mut received = Value::None;
        for _ in 0..200 {
            received = dispatch_method("read", &[accepted.clone(), int(2)])
                .unwrap()
                .unwrap();
            if !matches!(received, Value::None) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        let Value::Object(handle) = received else {
            panic!("Some attendu");
        };
        let Object::Option(Some(Value::Object(bytes_handle))) = &*handle.borrow() else {
            panic!("lecteur TCP doit fournir Some(List<int>)");
        };
        assert!(matches!(&*bytes_handle.borrow(), Object::Array(items) if !items.is_empty()));

        dispatch_method("close", std::slice::from_ref(&client)).unwrap();
        dispatch_method("close", std::slice::from_ref(&accepted)).unwrap();
        dispatch_method("close", std::slice::from_ref(&listener)).unwrap();
    }

    #[test]
    fn udp_send_and_receive_work_on_loopback() {
        let sender = native_udp_bind(&[string("127.0.0.1"), int(0)]).unwrap();
        let receiver = native_udp_bind(&[string("127.0.0.1"), int(0)]).unwrap();
        let receiver_address = dispatch_method("local_addr", std::slice::from_ref(&receiver))
            .unwrap()
            .unwrap();
        let (host, port) = parse_socket_address(&receiver_address);

        let sent = dispatch_method(
            "send_to",
            &[sender.clone(), bytes(&[1, 2, 3]), string(&host), int(port)],
        )
        .unwrap()
        .unwrap();
        assert_eq!(sent, Value::Integer(3));

        let mut packet = Value::None;
        for _ in 0..200 {
            packet = dispatch_method("recv_from", &[receiver.clone(), int(16)])
                .unwrap()
                .unwrap();
            if !matches!(packet, Value::None) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }

        assert!(matches!(packet, Value::Object(handle) if matches!(&*handle.borrow(), Object::Option(Some(Value::Object(tuple))) if matches!(&*tuple.borrow(), Object::Tuple(items) if items.len() == 3))));

        dispatch_method("close", std::slice::from_ref(&sender)).unwrap();
        dispatch_method("close", std::slice::from_ref(&receiver)).unwrap();
    }

    #[test]
    fn tcp_shutdown_nodelay_and_closed_state_are_stable() {
        let listener = native_tcp_listen(&[string("127.0.0.1"), int(0)]).unwrap();
        let address = dispatch_method("local_addr", std::slice::from_ref(&listener))
            .unwrap()
            .unwrap();
        let (host, port) = parse_socket_address(&address);
        let client = native_tcp_connect(&[string(&host), int(port)]).unwrap();

        let accepted = loop {
            let value = dispatch_method("accept", std::slice::from_ref(&listener))
                .unwrap()
                .unwrap();
            if matches!(value, Value::None) {
                std::thread::sleep(std::time::Duration::from_millis(1));
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

        assert_eq!(dispatch_method("nodelay", std::slice::from_ref(&client)).unwrap(), Some(Value::Boolean(false)));
        assert_eq!(dispatch_method("set_nodelay", &[client.clone(), Value::Boolean(true)]).unwrap(), Some(Value::None));
        assert_eq!(dispatch_method("nodelay", std::slice::from_ref(&client)).unwrap(), Some(Value::Boolean(true)));

        assert_eq!(dispatch_method("shutdown", &[client.clone(), string("write")]).unwrap(), Some(Value::None));
        assert_eq!(dispatch_method("is_closed", std::slice::from_ref(&client)).unwrap(), Some(Value::Boolean(false)));

        assert_eq!(dispatch_method("close", std::slice::from_ref(&client)).unwrap(), Some(Value::None));
        assert_eq!(dispatch_method("is_closed", std::slice::from_ref(&client)).unwrap(), Some(Value::Boolean(true)));
        assert!(matches!(
            dispatch_method("peer_addr", std::slice::from_ref(&client)),
            Err(RuntimeError::NetworkError { kind: "Closed", .. })
        ));

        dispatch_method("close", std::slice::from_ref(&accepted)).unwrap();
        dispatch_method("close", std::slice::from_ref(&listener)).unwrap();
    }

    #[test]
    fn udp_connect_send_and_peer_address_work() {
        let sender = native_udp_bind(&[string("127.0.0.1"), int(0)]).unwrap();
        let receiver = native_udp_bind(&[string("127.0.0.1"), int(0)]).unwrap();
        let address = dispatch_method("local_addr", std::slice::from_ref(&receiver))
            .unwrap()
            .unwrap();
        let (host, port) = parse_socket_address(&address);

        assert_eq!(
            dispatch_method("connect", &[sender.clone(), string(&host), int(port)]).unwrap(),
            Some(Value::None)
        );
        assert_eq!(
            dispatch_method("peer_addr", std::slice::from_ref(&sender)).unwrap(),
            Some(Value::new_string(address.as_string_value().unwrap()))
        );
        assert_eq!(
            dispatch_method("send", &[sender.clone(), bytes(&[7, 8, 9])]).unwrap(),
            Some(Value::Integer(3))
        );

        let mut packet = Value::None;
        for _ in 0..200 {
            packet = dispatch_method("recv_from", &[receiver.clone(), int(16)])
                .unwrap()
                .unwrap();
            if !matches!(packet, Value::None) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(matches!(packet, Value::Object(handle) if matches!(&*handle.borrow(), Object::Option(Some(Value::Object(tuple))) if matches!(&*tuple.borrow(), Object::Tuple(items) if items.len() == 3))));

        assert_eq!(dispatch_method("is_closed", std::slice::from_ref(&sender)).unwrap(), Some(Value::Boolean(false)));
        assert_eq!(dispatch_method("close", std::slice::from_ref(&sender)).unwrap(), Some(Value::None));
        assert_eq!(dispatch_method("is_closed", std::slice::from_ref(&sender)).unwrap(), Some(Value::Boolean(true)));
        dispatch_method("close", std::slice::from_ref(&receiver)).unwrap();
    }

    #[test]
    fn network_rejects_invalid_arguments_and_bytes() {
        assert!(matches!(native_tcp_connect(&[]), Err(RuntimeError::WrongArgumentCount { .. })));
        assert!(matches!(native_tcp_connect(&[string("127.0.0.1"), int(-1)]), Err(RuntimeError::TypeError)));
        assert!(matches!(native_tcp_listen(&[string("127.0.0.1"), int(65536)]), Err(RuntimeError::TypeError)));

        let listener = native_tcp_listen(&[string("127.0.0.1"), int(0)]).unwrap();
        assert!(matches!(
            dispatch_method("write", &[listener.clone(), bytes(&[1])]),
            Err(RuntimeError::ObjectFieldNotFound { .. })
        ));
        assert!(matches!(
            dispatch_method("read", &[listener.clone(), int(-1)]),
            Err(RuntimeError::ObjectFieldNotFound { .. })
        ));

        let address = dispatch_method("local_addr", std::slice::from_ref(&listener))
            .unwrap()
            .unwrap();
        let (host, port) = parse_socket_address(&address);
        let client = native_tcp_connect(&[string(&host), int(port)]).unwrap();
        let accepted = loop {
            let value = dispatch_method("accept", std::slice::from_ref(&listener))
                .unwrap()
                .unwrap();
            if matches!(value, Value::None) {
                std::thread::sleep(std::time::Duration::from_millis(1));
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

        assert!(matches!(
            dispatch_method("write", &[client.clone(), bytes(&[300])]),
            Err(RuntimeError::TypeError)
        ));
        assert!(matches!(
            dispatch_method("read", &[accepted.clone(), int(-1)]),
            Err(RuntimeError::TypeError)
        ));

        dispatch_method("close", std::slice::from_ref(&client)).unwrap();
        dispatch_method("close", std::slice::from_ref(&accepted)).unwrap();
        dispatch_method("close", std::slice::from_ref(&listener)).unwrap();
    }
}
