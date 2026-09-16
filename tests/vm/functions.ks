// VM - functions
func add(a, b) {
    return a + b;
}

func greet(name) {
    return "Hello " + name;
}

func no_return() {
    println("inside");
}

println(add(10, 20));
println(greet("Kastel"));
no_return();
