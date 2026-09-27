func f(value) -> int {
    match value {
        Some(x) => { return x; }
        None => { return 0; }
    }
}


let i = f(Some(45))

println(i)