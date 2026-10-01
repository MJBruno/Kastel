// Variable de condition associée à un Mutex.

export func create(mutex: Mutex) -> Condvar {
    return condvar(mutex);
}
