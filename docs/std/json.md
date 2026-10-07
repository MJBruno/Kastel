# std.json

Encodage et décodage JSON sans dépendance externe.

## API

- `encode(value: dynamic) -> Result<str, str>`
- `decode(text: str) -> Result<dynamic, str>`
- `read_file(path: str) -> Result<dynamic, str>`
- `write_file(path: str, value: dynamic) -> Result<bool, str>`
- `get_path(data: dynamic, path: str) -> Option<dynamic>`

Les erreurs de syntaxe, de type et d'I/O sont explicites via `Result`.
