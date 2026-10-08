# `std.net`

`std.net` fournit les primitives réseau de Kastel inspirées de `std::net` de Rust : TCP, UDP, adresses IP, adresses de socket et résolution DNS.

## Différence fondamentale avec Rust

Kastel utilise des sockets **non bloquantes par défaut** afin de préserver le scheduler coopératif de la VM.

Une opération qui ne peut pas avancer immédiatement renvoie `None` pour `read`, `peek`, `recv`, `recv_from` et `accept`. Pour les opérations d'écriture, un retour `0` indique qu'aucun octet n'a été écrit immédiatement.

`set_nonblocking(false)` reste disponible pour les usages avancés, mais peut bloquer la VM et doit donc être évité dans le code coopératif normal.

## Adresses

Les valeurs suivantes sont natives :

- `IpAddr`
- `Ipv4Addr`
- `Ipv6Addr`
- `SocketAddr`
- `SocketAddrV4`
- `SocketAddrV6`

Constructeurs principaux :

```ks
let ip = ipv4(127, 0, 0, 1);
let any = ipv6(0, 0, 0, 0, 0, 0, 0, 1);
let address = socket_addr(ip.to_ip(), 8080);

let localhost = ipv4_localhost();
let unspecified = ipv6_unspecified();
```

`ipv4_from_bits()` complète `Ipv4Addr::from_bits()` de Rust.

Les parseurs de haut niveau renvoient `Result` :

```ks
from std.net import parse_ip, parse_socket_addr

let ip = parse_ip("127.0.0.1");
let address = parse_socket_addr("127.0.0.1:8080");
```

## Résolution

Kastel remplace le trait Rust `ToSocketAddrs` par des fonctions explicites :

```ks
from std.net import resolve_host, resolve_socket_addr

let addresses = resolve_host("localhost", 8080);
let parsed = resolve_socket_addr("localhost:8080");
```

Cela respecte le modèle dynamique/graduel de Kastel sans imposer un mécanisme de traits aux chaînes et tuples.

## TCP

```ks
from std.net import listen_tcp, connect_tcp

let server = listen_tcp("127.0.0.1", 0);
let client = connect_tcp("127.0.0.1", 8080);
```

`TcpStream` expose notamment :

```text
read(size)
peek(size)
write(bytes)
shutdown("read" | "write" | "both")
set_nodelay(bool)
nodelay()
set_ttl(int)
ttl()
set_nonblocking(bool)
is_nonblocking()
set_read_timeout(Option<int>)
read_timeout()
set_write_timeout(Option<int>)
write_timeout()
take_error()
try_clone()
local_addr()
local_socket_addr()
peer_addr()
peer_socket_addr()
is_closed()
close()
```

`TcpListener` expose :

```text
accept()
accept_available(max)
try_clone()
set_ttl(int)
ttl()
set_nonblocking(bool)
is_nonblocking()
take_error()
local_addr()
local_socket_addr()
is_closed()
close()
```

`accept_available(max)` est l'adaptation Kastel de l'idée `incoming()` de Rust : il draine au plus `max` connexions immédiatement disponibles au lieu de créer un itérateur bloquant.

## UDP

`UdpSocket` expose :

```text
send_to(bytes, host, port)
send_to_addr(bytes, SocketAddr)
recv_from(size)
recv_from_addr(size)
peek(size)
peek_from(size)
connect(host, port)
connect_addr(SocketAddr)
send(bytes)
recv(size)
peer_addr()
peer_socket_addr()
local_addr()
local_socket_addr()
try_clone()
set_nonblocking(bool)
is_nonblocking()
set_read_timeout(Option<int>)
read_timeout()
set_write_timeout(Option<int>)
write_timeout()
take_error()
set_broadcast(bool)
broadcast()
set_ttl(int)
ttl()
set_multicast_loop_v4(bool)
multicast_loop_v4()
set_multicast_ttl_v4(int)
multicast_ttl_v4()
set_multicast_loop_v6(bool)
multicast_loop_v6()
join_multicast_v4(Ipv4Addr, Ipv4Addr)
leave_multicast_v4(Ipv4Addr, Ipv4Addr)
join_multicast_v6(Ipv6Addr, int)
leave_multicast_v6(Ipv6Addr, int)
is_closed()
close()
```

## Timeout

Les durées sont représentées en millisecondes :

```ks
socket.set_read_timeout(Some(1000));
```

`None` retire le timeout. Les réglages suivent les primitives de `std::net`; ils ne désactivent pas automatiquement le mode non bloquant.

## Ce qui n'est pas exposé

Les API Rust instables ou non disponibles dans `std::net` stable ne sont pas simulées : notamment `TcpStream::set_keepalive`, `TcpStream::set_linger` et les extensions système spécifiques.

Kastel n'intègre pas non plus un faux `write_all` : avec des sockets non bloquantes, une boucle d'écriture interne bloquante serait incompatible avec le scheduler coopératif. La construction correcte d'un `write_all` coopératif appartient à une future couche de reactor/IO readiness.
