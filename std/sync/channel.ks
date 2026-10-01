// Canal coopératif.
// Le type d'élément peut être précisé à l'utilisation :
// `let c: Channel<int> = channel.create();`

export func create() {
    return channel();
}

export func create_bounded(capacity: int) {
    return channel(capacity);
}
