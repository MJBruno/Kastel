// std/thread.ks — P5
// API coopérative officielle : tâches et sélection typée.

export type SelectResult<T> = {
    index: int,
    value: Option<T>,
    closed: bool
};

export func spawn_task<T>(task: func() -> T) -> Task<T> {
    return spawn(task);
}

export func spawn_task<A, T>(task: func(A) -> T, arg: A) -> Task<T> {
    return spawn(task, arg);
}

export func spawn_task<A, B, T>(task: func(A, B) -> T, first: A, second: B) -> Task<T> {
    return spawn(task, first, second);
}

export func spawn_task<A, B, C, T>(
    task: func(A, B, C) -> T,
    first: A,
    second: B,
    third: C
) -> Task<T> {
    return spawn(task, first, second, third);
}

// Sélection réceptionnelle typée.
// - index >= 0 et closed == false : une valeur T a été reçue.
// - closed == true : le canal sélectionné est fermé et value == None.
// - index == -1 : timeout, avec value == None.
export func select_channels<T>(channels: List<Channel<T>>) -> SelectResult<T> {
    let raw = select(channels);
    let index = raw[0];
    let closed = raw[2];

    if index < 0 || closed {
        return { index: index, value: None, closed: closed };
    }

    return { index: index, value: Some<T>(raw[1]), closed: false };
}

export func select_channels<T>(channels: List<Channel<T>>, timeout_ms: int) -> SelectResult<T> {
    let raw = select(channels, timeout_ms);
    let index = raw[0];
    let closed = raw[2];

    if index < 0 || closed {
        return { index: index, value: None, closed: closed };
    }

    return { index: index, value: Some<T>(raw[1]), closed: false };
}

export func yield_now() -> None {
    yield();
}

export func sleep_ms(milliseconds: int) -> None {
    sleep(milliseconds);
}

// Opérations fonctionnelles typées sur Task<T>.
// Les méthodes natives restent disponibles directement sur la valeur Task<T>.
export func join_task<T>(task: Task<T>) -> T {
    return task.join();
}

export func cancel_task<T>(task: Task<T>) -> None {
    task.cancel();
}

export func task_status<T>(task: Task<T>) -> str {
    return task.status();
}

export func task_done<T>(task: Task<T>) -> bool {
    return task.is_done();
}


// Exécute une action sous protection d'un Mutex et libère toujours le verrou,
// même lorsque l'action lève une exception.
export func with_lock<T>(mutex: Mutex, action: func() -> T) -> T {
    mutex.lock();
    try {
        return action();
    } finally {
        mutex.unlock();
    }
}

// Exécute une action sous verrou de lecture RwLock.
export func with_read_lock<T>(lock: RwLock, action: func() -> T) -> T {
    lock.read_lock();
    try {
        return action();
    } finally {
        lock.read_unlock();
    }
}

// Exécute une action sous verrou d'écriture RwLock.
export func with_write_lock<T>(lock: RwLock, action: func() -> T) -> T {
    lock.write_lock();
    try {
        return action();
    } finally {
        lock.write_unlock();
    }
}

// Acquiert temporairement un permis de Semaphore et le restitue toujours.
export func with_semaphore<T>(semaphore: Semaphore, action: func() -> T) -> T {
    semaphore.acquire();
    try {
        return action();
    } finally {
        semaphore.release();
    }
}
