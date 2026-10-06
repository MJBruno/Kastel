use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_NETWORK_ID: AtomicUsize = AtomicUsize::new(1);

fn next_id() -> usize {
    NEXT_NETWORK_ID.fetch_add(1, Ordering::Relaxed)
}

/// État opaque des ressources réseau détenues par le runtime.
///
/// Les sockets sont non bloquants : une lecture/réception ou un `accept()`
/// qui n'a rien à fournir immédiatement retourne `None` côté Kastel au lieu
/// de bloquer toute la VM et son scheduler coopératif.
#[derive(Debug)]
pub enum NetworkState {
    TcpStream {
        id: usize,
        socket: Option<TcpStream>,
    },
    TcpListener {
        id: usize,
        socket: Option<TcpListener>,
    },
    UdpSocket {
        id: usize,
        socket: Option<UdpSocket>,
    },
}

impl NetworkState {
    pub fn tcp_stream(socket: TcpStream) -> Self {
        Self::TcpStream {
            id: next_id(),
            socket: Some(socket),
        }
    }

    pub fn tcp_listener(socket: TcpListener) -> Self {
        Self::TcpListener {
            id: next_id(),
            socket: Some(socket),
        }
    }

    pub fn udp_socket(socket: UdpSocket) -> Self {
        Self::UdpSocket {
            id: next_id(),
            socket: Some(socket),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::TcpStream { .. } => "TcpStream",
            Self::TcpListener { .. } => "TcpListener",
            Self::UdpSocket { .. } => "UdpSocket",
        }
    }

    pub fn display_name(&self) -> &'static str {
        self.type_name()
    }

    pub fn id(&self) -> usize {
        match self {
            Self::TcpStream { id, .. }
            | Self::TcpListener { id, .. }
            | Self::UdpSocket { id, .. } => *id,
        }
    }

    pub fn close(&mut self) -> bool {
        match self {
            Self::TcpStream { socket, .. } => socket.take().is_some(),
            Self::TcpListener { socket, .. } => socket.take().is_some(),
            Self::UdpSocket { socket, .. } => socket.take().is_some(),
        }
    }

    pub fn is_closed(&self) -> bool {
        match self {
            Self::TcpStream { socket, .. } => socket.is_none(),
            Self::TcpListener { socket, .. } => socket.is_none(),
            Self::UdpSocket { socket, .. } => socket.is_none(),
        }
    }
}

impl PartialEq for NetworkState {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id() && self.type_name() == other.type_name()
    }
}

impl Eq for NetworkState {}
