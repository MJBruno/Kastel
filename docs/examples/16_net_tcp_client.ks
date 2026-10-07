import std.net;

match net.connect_tcp("127.0.0.1", 8080) {
    Ok(client) => {
        println(client.peer_addr());
        println(client.local_addr());

        let written = client.write([72, 105, 10]);
        println(written);

        let attempts = 0;
        while attempts < 200 {
            match client.read(1024) {
                Some(bytes) => {
                    println(bytes);
                    client.close();
                    break;
                }
                None => {
                    attempts = attempts + 1;
                }
            }
        }

        println(client.is_closed());
    }
    Err(error) => println(error);
}

println("std.net TCP client: OK");
