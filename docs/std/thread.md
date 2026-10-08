# std.thread

Contrôle des tâches coopératives du scheduler Kastel.

## API

- `spawn_task(task: dynamic) -> Task<dynamic>`
- `yield_now() -> None`
- `sleep_ms(milliseconds: int) -> None`

Pour les canaux, verrous et primitives d'attente, utiliser `std.sync.*`.

```Rust

import std.thread

let a = thread.spawn_task(func() {
        return 10;
});

let b = thread.spawn_task(func(x) {
        return x * 2;
    }, 21);

let c = thread.spawn_task(func(x, y) {
        return x + y;
    }, 10, 20);

println(await a);
println(await b);
println(await c);
```

```Rust
import std.thread
from std.thread import SelectResult

let a: Channel<int> = channel();
let b: Channel<int> = channel();
b.send(42);

let result: SelectResult<int> = thread.select_channels([a, b]);
println(result.index);
println(result.value.unwrap());
println(result.closed);

let timeout_result: SelectResult<int> = thread.select_channels([a], 0);
println(timeout_result.index);
println(timeout_result.value.is_none());

```
