# std.net

TCP/UDP officiels. Les constructeurs renvoient `Result` afin de rendre les erreurs de création récupérables.

```text
connect_tcp(host: str, port: int) -> Result<TcpStream, str>
listen_tcp(host: str, port: int) -> Result<TcpListener, str>
bind_udp(host: str, port: int) -> Result<UdpSocket, str>
```

Les lectures et `accept()` restent non bloquants au niveau runtime.
