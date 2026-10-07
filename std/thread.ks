// std/thread.ks
// API coopérative officielle.

export func spawn_task<T>(task: func() -> T) -> Task<T> {
    return spawn(task);
}

export func yield_now() -> None {
    yield();
}

export func sleep_ms(milliseconds: int) -> None {
    sleep(milliseconds);
}
