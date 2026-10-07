import std.http;

match http.get("http://example.com/") {
    Ok(response) => {
        println(response.status);
        println(response.version);
        println(response.reason);
        println(response.headers);
        println(response.body.size());
    }
    Err(error) => println(error);
}

let headers = {
    "Accept": "text/plain",
    "User-Agent": "Kastel/1.0"
};

match http.request(
    "GET",
    "http://example.com/",
    headers,
    None
) {
    Ok(response) => println(response.status);
    Err(error) => println(error);
}

println("std.http: OK");
