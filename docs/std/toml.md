# std.toml

Lecteur et générateur TOML intégré à `std`, sans dépendance externe.

La version 1.0 expose volontairement un sous-ensemble documenté. Les fonctionnalités non prises en charge doivent produire une erreur déterministe plutôt qu'une interprétation silencieuse.

## API

- `parse(text: str) -> Result<Dict<str, dynamic>, str>`
- `stringify(data: Dict<str, dynamic>) -> Result<str, str>`
- `read_file(path: str) -> Result<Dict<str, dynamic>, str>`
- `write_file(path: str, data: Dict<str, dynamic>) -> Result<bool, str>`
