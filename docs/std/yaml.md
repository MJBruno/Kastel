# std.yaml

Analyseur et générateur YAML intégré à `std`, sans dépendance externe.

La version 1.0 couvre le sous-ensemble documenté par le module et refuse explicitement les constructions hors périmètre.

## API

- `parse(text: str) -> Result<dynamic, str>`
- `stringify(value: dynamic) -> Result<str, str>`
- `read_file(path: str) -> Result<dynamic, str>`
- `write_file(path: str, value: dynamic) -> Result<bool, str>`
