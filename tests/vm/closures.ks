// VM - closures / upvalues
func make_counter() {
    let count = 0;

    func next() {
        count = count + 1;
        return count;
    }

    return next;
}

let counter = make_counter();
println(counter());
println(counter());
println(counter());
