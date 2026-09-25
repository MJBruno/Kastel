func  add<T:Add>(a:T, b:T)->T {
    return a+b
}

let a: int = add(5,6)

println(a)