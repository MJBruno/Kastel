// Sémaphore coopératif.

export func create(capacity: int) -> Semaphore {
    return semaphore(capacity);
}
