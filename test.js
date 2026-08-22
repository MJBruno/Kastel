
let name = "Bruno"
name = "John"

function test(a) {
    let b = 20

    {
        let c = 30
        return a + b + c
    }
}

let result = test(10)
print(result)
print(name)