func identity<T>(value: T) -> T {
    return value;
}

let value = identity<int, str>(10);
