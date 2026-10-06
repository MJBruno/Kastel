# std.csv

CSV RFC 4180-style avec champs quotés, guillemets doublés et retours `\n`/`\r\n`.

```text
parse(text: str, delimiter: str) -> Result<List<List<str>>, str>
parse_default(text: str) -> Result<List<List<str>>, str>
parse_with_header(text: str, delimiter: str) -> Result<List<Dict<str,str>>, str>
stringify(rows: List<List<dynamic>>, delimiter: str) -> Result<str, str>
read_file(path: str, delimiter: str) -> Result<List<List<str>>, str>
write_file(path: str, rows: List<List<dynamic>>, delimiter: str) -> Result<bool, str>
```

Le séparateur doit être un caractère unique différent de `"`, `\n` et `\r`.
