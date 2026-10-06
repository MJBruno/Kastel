import std.net;

let listener: Result<TcpListener, str> = net.listen_tcp("127.0.0.1", 8080);
let socket: Result<TcpStream, str> = net.connect_tcp("127.0.0.1", 8080);
let udp: Result<UdpSocket, str> = net.bind_udp("127.0.0.1", 0);

match listener {
    Ok(x) => println("ok"),
    Err(x) => println("Erreur"),
}
