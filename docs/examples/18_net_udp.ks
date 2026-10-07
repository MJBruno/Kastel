import std.net;

match net.bind_udp("127.0.0.1", 0) {
    Ok(sender) => {
        match net.bind_udp("127.0.0.1", 0) {
            Ok(receiver) => {
                println(sender.local_addr());
                println(receiver.local_addr());

                let address = receiver.local_addr();
                let host = "127.0.0.1";
                let port_start = address.index_of(":");
                let port = address.substring(port_start + 1, address.size()).to_int();

                println(sender.send_to([1, 2, 3], host, port));

                let attempts = 0;
                while attempts < 200 {
                    match receiver.recv_from(1024) {
                        Some(packet) => {
                            println(packet);
                            break;
                        }
                        None => {
                            attempts = attempts + 1;
                        }
                    }
                }

                sender.close();
                receiver.close();
            }
            Err(error) => println(error);
        }
    }
    Err(error) => println(error);
}

println("std.net UDP: OK");
