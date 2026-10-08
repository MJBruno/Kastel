// std/net.ks — couche réseau standard de Kastel.
//
// API adaptée de std::net de Rust :
// - résolution d'adresses et types IP/SocketAddr ;
// - TCP client/serveur ;
// - UDP et multicast ;
// - opérations non bloquantes ;
// - Result pour les opérations de construction/résolution.
//
// Les sockets créées par std.net sont non bloquantes afin de ne pas bloquer
// la VM coopérative. `read`, `recv`, `peek` et `accept` renvoient `None`
// lorsque l'opération ne peut pas avancer immédiatement.

export func resolve_host(host: str, port: int) -> Result<List<SocketAddr>, str> {
    try {
        return Ok(resolve(host, port));
    } catch (error) {
        return Err(error);
    }
}

export func resolve_socket_addr(address: str) -> Result<List<SocketAddr>, str> {
    try {
        return Ok(resolve_addr(address));
    } catch (error) {
        return Err(error);
    }
}

export func parse_ip(text: str) -> Result<IpAddr, str> {
    try {
        return Ok(ip_parse(text));
    } catch (error) {
        return Err(error);
    }
}

export func parse_socket_addr(text: str) -> Result<SocketAddr, str> {
    try {
        return Ok(socket_addr_parse(text));
    } catch (error) {
        return Err(error);
    }
}

export func connect_tcp(host: str, port: int) -> Result<TcpStream, str> {
    try {
        return Ok(tcp_connect(host, port));
    } catch (error) {
        return Err(error);
    }
}

export func connect_tcp_addr(address: SocketAddr) -> Result<TcpStream, str> {
    try {
        return Ok(tcp_connect_addr(address));
    } catch (error) {
        return Err(error);
    }
}

export func connect_tcp_timeout(address: SocketAddr, timeout_ms: int) -> Result<TcpStream, str> {
    try {
        return Ok(tcp_connect_timeout(address, timeout_ms));
    } catch (error) {
        return Err(error);
    }
}

export func listen_tcp(host: str, port: int) -> Result<TcpListener, str> {
    try {
        return Ok(tcp_listen(host, port));
    } catch (error) {
        return Err(error);
    }
}

export func listen_tcp_addr(address: SocketAddr) -> Result<TcpListener, str> {
    try {
        return Ok(tcp_listen_addr(address));
    } catch (error) {
        return Err(error);
    }
}

export func bind_udp(host: str, port: int) -> Result<UdpSocket, str> {
    try {
        return Ok(udp_bind(host, port));
    } catch (error) {
        return Err(error);
    }
}

export func bind_udp_addr(address: SocketAddr) -> Result<UdpSocket, str> {
    try {
        return Ok(udp_bind_addr(address));
    } catch (error) {
        return Err(error);
    }
}
