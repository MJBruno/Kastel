# std.http

Client HTTP/1.1 officiel au-dessus de `std.net`.

```text
type HttpResponse = {
    status: int,
    headers: Dict<str, str>,
    body: List<int>,
    version: str,
    reason: str
}

get(url: str) -> Result<HttpResponse, str>
request(method: str, url: str, headers: Dict<str, str>, body: dynamic) -> Result<HttpResponse, str>
```

`https://` n'est pas accepté par `std.http 1.0`; TLS doit être ajouté comme couche explicitement dédiée.
