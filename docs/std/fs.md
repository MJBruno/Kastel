# std.fs

API officielle des fichiers. Les opérations pouvant échouer renvoient `Result`.

```text
read_text(path: str) -> Result<str, str>
read_lines(path: str) -> Result<List<str>, str>
write_text(path: str, content: str) -> Result<bool, str>
append_text(path: str, content: str) -> Result<bool, str>
delete_file(path: str) -> Result<bool, str>
size_of(path: str) -> Result<int, str>
exists(path: str) -> bool
read_text_or(path: str, fallback: str) -> str
```

Les chemins sont gérés exclusivement par `std.path`.
