// API coopérative officielle.
//
// `spawn_task<T>` utilise un callback sans argument et conserve le type de
// résultat à travers le Task<T>. Les fonctions `async` sont volontairement
// exclues par le type de fonction : elles produisent déjà un Task<T> à l'appel.

export func spawn_task<T>(task: func() -> T) -> Task<T> {
    return spawn(task);
}

export func yield_now() -> None {
    yield();
}

export func sleep_ms(milliseconds: int) -> None {
    sleep(milliseconds);
}
