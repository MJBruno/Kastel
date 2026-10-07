# std.process

Exécution de processus sans shell implicite.

## API

- `run(program: str, arguments: List<str>) -> Result<ProcessOutput, str>`
- `run0(program: str) -> Result<ProcessOutput, str>`
- `run_checked(program: str, arguments: List<str>) -> Result<ProcessOutput, str>`

`ProcessOutput` fournit `stdout`, `stderr`, `code` et `success`.
