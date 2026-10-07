import std.net;
import std.thread;

match net.listen_tcp("127.0.0.1", 8080) {
    Ok(listener) => {
        println("TCP server: 127.0.0.1:8080");
        println(listener.local_addr());
        println("Ctrl+C pour arreter");

        while true {
            match listener.accept() {
                Some(client) => {
                    let attempts = 0;
                    while attempts < 200 {
                        match client.read(1024) {
                            Some(bytes) => {
                                println(bytes);
                                client.write([75, 97, 115, 116, 101, 108, 10]);
                                client.close();
                                break;
                            }
                            None => {
                                attempts = attempts + 1;
                                thread.yield_now();
                            }
                        }
                    }
                }
                None => {
                    thread.yield_now();
                }
            }
        }
    }
    Err(error) => println(error);
}
