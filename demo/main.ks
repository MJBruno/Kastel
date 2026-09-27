func f(value: Option<int>) -> int {
    match value {
        Some(x) => { return x; }
        None => { return 0; }
    }
}