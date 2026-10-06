// Canal coopératif générique.

export func create<T>() -> Channel<T> {
    return channel<T>();
}

export func create_bounded<T>(capacity: int) -> Channel<T> {
    return channel<T>(capacity);
}
