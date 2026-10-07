# std.thread

Contrôle des tâches coopératives du scheduler Kastel.

## API

- `spawn_task(task: dynamic) -> Task<dynamic>`
- `yield_now() -> None`
- `sleep_ms(milliseconds: int) -> None`

Pour les canaux, verrous et primitives d'attente, utiliser `std.sync.*`.
