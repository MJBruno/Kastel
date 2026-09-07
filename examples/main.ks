println("TEST IDENTITE CLOSURES");

function make_counter() {
    let count = 0;

    function increment() {
        count += 1;
        return count;
    }

    return increment;
}

let a = make_counter();
let b = make_counter();
let c = a;

println(a == b);
println(a == c);

println(a());
println(b());
println(a());
println(b());