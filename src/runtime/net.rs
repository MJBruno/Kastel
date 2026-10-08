use std::cell::Cell;
use std::net::{
    IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, TcpListener,
    TcpStream, UdpSocket,
};
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_NETWORK_ID: AtomicUsize = AtomicUsize::new(1);

fn next_id() -> usize {
    NEXT_NETWORK_ID.fetch_add(1, Ordering::Relaxed)
}

/// Ressources et valeurs réseau détenues par le runtime.
///
/// Les trois sockets sont manipulés comme des ressources possédées par la VM.
/// Les adresses sont également stockées ici pour réutiliser l'enveloppe
/// `Object::Network` existante sans introduire un nouveau type d'objet GC.
#[derive(Debug)]
pub enum NetworkState {
    TcpStream {
        id: usize,
        socket: Option<TcpStream>,
        nonblocking: Rc<Cell<bool>>,
    },
    TcpListener {
        id: usize,
        socket: Option<TcpListener>,
        nonblocking: Rc<Cell<bool>>,
    },
    UdpSocket {
        id: usize,
        socket: Option<UdpSocket>,
        nonblocking: Rc<Cell<bool>>,
    },
    IpAddr {
        id: usize,
        address: IpAddr,
    },
    Ipv4Addr {
        id: usize,
        address: Ipv4Addr,
    },
    Ipv6Addr {
        id: usize,
        address: Ipv6Addr,
    },
    SocketAddr {
        id: usize,
        address: SocketAddr,
    },
    SocketAddrV4 {
        id: usize,
        address: SocketAddrV4,
    },
    SocketAddrV6 {
        id: usize,
        address: SocketAddrV6,
    },
}

impl NetworkState {
    pub fn tcp_stream(socket: TcpStream) -> Self {
        Self::TcpStream {
            id: next_id(),
            socket: Some(socket),
            nonblocking: Rc::new(Cell::new(true)),
        }
    }

    pub fn tcp_stream_with_nonblocking(socket: TcpStream, nonblocking: Rc<Cell<bool>>) -> Self {
        Self::TcpStream {
            id: next_id(),
            socket: Some(socket),
            nonblocking,
        }
    }

    pub fn tcp_listener(socket: TcpListener) -> Self {
        Self::TcpListener {
            id: next_id(),
            socket: Some(socket),
            nonblocking: Rc::new(Cell::new(true)),
        }
    }

    pub fn tcp_listener_with_nonblocking(socket: TcpListener, nonblocking: Rc<Cell<bool>>) -> Self {
        Self::TcpListener {
            id: next_id(),
            socket: Some(socket),
            nonblocking,
        }
    }

    pub fn udp_socket(socket: UdpSocket) -> Self {
        Self::UdpSocket {
            id: next_id(),
            socket: Some(socket),
            nonblocking: Rc::new(Cell::new(true)),
        }
    }

    pub fn udp_socket_with_nonblocking(socket: UdpSocket, nonblocking: Rc<Cell<bool>>) -> Self {
        Self::UdpSocket {
            id: next_id(),
            socket: Some(socket),
            nonblocking,
        }
    }

    pub fn ip_addr(address: IpAddr) -> Self {
        Self::IpAddr {
            id: next_id(),
            address,
        }
    }

    pub fn ipv4_addr(address: Ipv4Addr) -> Self {
        Self::Ipv4Addr {
            id: next_id(),
            address,
        }
    }

    pub fn ipv6_addr(address: Ipv6Addr) -> Self {
        Self::Ipv6Addr {
            id: next_id(),
            address,
        }
    }

    pub fn socket_addr(address: SocketAddr) -> Self {
        Self::SocketAddr {
            id: next_id(),
            address,
        }
    }

    pub fn socket_addr_v4(address: SocketAddrV4) -> Self {
        Self::SocketAddrV4 {
            id: next_id(),
            address,
        }
    }

    pub fn socket_addr_v6(address: SocketAddrV6) -> Self {
        Self::SocketAddrV6 {
            id: next_id(),
            address,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::TcpStream { .. } => "TcpStream",
            Self::TcpListener { .. } => "TcpListener",
            Self::UdpSocket { .. } => "UdpSocket",
            Self::IpAddr { .. } => "IpAddr",
            Self::Ipv4Addr { .. } => "Ipv4Addr",
            Self::Ipv6Addr { .. } => "Ipv6Addr",
            Self::SocketAddr { .. } => "SocketAddr",
            Self::SocketAddrV4 { .. } => "SocketAddrV4",
            Self::SocketAddrV6 { .. } => "SocketAddrV6",
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            Self::TcpStream { .. } => "TcpStream".into(),
            Self::TcpListener { .. } => "TcpListener".into(),
            Self::UdpSocket { .. } => "UdpSocket".into(),
            Self::IpAddr { address, .. } => address.to_string(),
            Self::Ipv4Addr { address, .. } => address.to_string(),
            Self::Ipv6Addr { address, .. } => address.to_string(),
            Self::SocketAddr { address, .. } => address.to_string(),
            Self::SocketAddrV4 { address, .. } => address.to_string(),
            Self::SocketAddrV6 { address, .. } => address.to_string(),
        }
    }

    pub fn id(&self) -> usize {
        match self {
            Self::TcpStream { id, .. }
            | Self::TcpListener { id, .. }
            | Self::UdpSocket { id, .. }
            | Self::IpAddr { id, .. }
            | Self::Ipv4Addr { id, .. }
            | Self::Ipv6Addr { id, .. }
            | Self::SocketAddr { id, .. }
            | Self::SocketAddrV4 { id, .. }
            | Self::SocketAddrV6 { id, .. } => *id,
        }
    }

    pub fn close(&mut self) -> bool {
        match self {
            Self::TcpStream { socket, .. } => socket.take().is_some(),
            Self::TcpListener { socket, .. } => socket.take().is_some(),
            Self::UdpSocket { socket, .. } => socket.take().is_some(),
            Self::IpAddr { .. }
            | Self::Ipv4Addr { .. }
            | Self::Ipv6Addr { .. }
            | Self::SocketAddr { .. }
            | Self::SocketAddrV4 { .. }
            | Self::SocketAddrV6 { .. } => false,
        }
    }

    pub fn is_closed(&self) -> bool {
        match self {
            Self::TcpStream { socket, .. } => socket.is_none(),
            Self::TcpListener { socket, .. } => socket.is_none(),
            Self::UdpSocket { socket, .. } => socket.is_none(),
            Self::IpAddr { .. }
            | Self::Ipv4Addr { .. }
            | Self::Ipv6Addr { .. }
            | Self::SocketAddr { .. }
            | Self::SocketAddrV4 { .. }
            | Self::SocketAddrV6 { .. } => false,
        }
    }

    /// Hachage cohérent avec `PartialEq` lorsqu'une adresse réseau est une
    /// clé de `Dict`/`Set`. Les sockets utilisent leur identité de ressource,
    /// les adresses leur contenu.
    pub fn hash_key(&self, state: &mut impl std::hash::Hasher) {
        use std::hash::Hash;

        match self {
            Self::TcpStream { id, .. } => {
                0u8.hash(state);
                id.hash(state);
            }
            Self::TcpListener { id, .. } => {
                1u8.hash(state);
                id.hash(state);
            }
            Self::UdpSocket { id, .. } => {
                2u8.hash(state);
                id.hash(state);
            }
            Self::IpAddr { address, .. } => {
                3u8.hash(state);
                address.hash(state);
            }
            Self::Ipv4Addr { address, .. } => {
                4u8.hash(state);
                address.hash(state);
            }
            Self::Ipv6Addr { address, .. } => {
                5u8.hash(state);
                address.hash(state);
            }
            Self::SocketAddr { address, .. } => {
                6u8.hash(state);
                address.hash(state);
            }
            Self::SocketAddrV4 { address, .. } => {
                7u8.hash(state);
                address.hash(state);
            }
            Self::SocketAddrV6 { address, .. } => {
                8u8.hash(state);
                address.hash(state);
            }
        }
    }

    pub fn is_socket(&self) -> bool {
        matches!(
            self,
            Self::TcpStream { .. } | Self::TcpListener { .. } | Self::UdpSocket { .. }
        )
    }
}

impl PartialEq for NetworkState {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::TcpStream { id: left, .. }, Self::TcpStream { id: right, .. })
            | (Self::TcpListener { id: left, .. }, Self::TcpListener { id: right, .. })
            | (Self::UdpSocket { id: left, .. }, Self::UdpSocket { id: right, .. }) => left == right,
            (Self::IpAddr { address: left, .. }, Self::IpAddr { address: right, .. }) => left == right,
            (Self::Ipv4Addr { address: left, .. }, Self::Ipv4Addr { address: right, .. }) => left == right,
            (Self::Ipv6Addr { address: left, .. }, Self::Ipv6Addr { address: right, .. }) => left == right,
            (Self::SocketAddr { address: left, .. }, Self::SocketAddr { address: right, .. }) => left == right,
            (Self::SocketAddrV4 { address: left, .. }, Self::SocketAddrV4 { address: right, .. }) => left == right,
            (Self::SocketAddrV6 { address: left, .. }, Self::SocketAddrV6 { address: right, .. }) => left == right,
            _ => false,
        }
    }
}

impl Eq for NetworkState {}
