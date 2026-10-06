// std/net.ks
// Ressources TCP/UDP officielles. Les sockets de lecture restent non bloquantes.

export func connect_tcp(host: str, port: int) -> Result<TcpStream, str> {
    try {
        return Ok(tcp_connect(host, port));
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

export func bind_udp(host: str, port: int) -> Result<UdpSocket, str> {
    try {
        return Ok(udp_bind(host, port));
    } catch (error) {
        return Err(error);
    }
}
