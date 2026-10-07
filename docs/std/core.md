# std.core

Utilitaires fondamentaux pour l'inspection de valeurs et les opérations simples.

## API

- `is_int(value: dynamic) -> bool`
- `is_float(value: dynamic) -> bool`
- `is_number(value: dynamic) -> bool`
- `is_bool(value: dynamic) -> bool`
- `is_string(value: dynamic) -> bool`
- `is_list(value: dynamic) -> bool`
- `is_dict(value: dynamic) -> bool`
- `is_set(value: dynamic) -> bool`
- `is_tuple(value: dynamic) -> bool`
- `is_none(value: dynamic) -> bool`
- `is_function(value: dynamic) -> bool`
- `type_name(value: dynamic) -> str`
- `require(condition: bool, message: str) -> None`
- `panic(message: str) -> None`
- `identity<T>(value: T) -> T`
- `constant<T>(value: T) -> dynamic`
- `compose(f: dynamic, g: dynamic) -> dynamic`
- `pipe(value: dynamic, steps: List<dynamic>) -> dynamic`
- `repeat(times: int, action: dynamic) -> None`
