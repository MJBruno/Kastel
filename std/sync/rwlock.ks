// Verrou lecture/écriture coopératif.

export func create() -> RwLock {
    return rwlock();
}
