function double(x) {
    return x + x
}

function test(a) {
    return double(a) * 5
}

let x = test(10)

print(x)