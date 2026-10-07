// std/http.ks
// Client HTTP/1.1 officiel. HTTPS/TLS est volontairement hors de std.http 1.0.

export type HttpResponse = {
    status: int,
    headers: Dict<str, str>,
    body: List<int>,
    version: str,
    reason: str
};

export func get(url: str) -> Result<HttpResponse, str> {
    try {
        return Ok(http_get(url));
    } catch (error) {
        return Err(error);
    }
}

export func request(
    method: str,
    url: str,
    headers: Dict<str, str>,
    body: dynamic
) -> Result<HttpResponse, str> {
    try {
        return Ok(http_request(method, url, headers, body));
    } catch (error) {
        return Err(error);
    }
}