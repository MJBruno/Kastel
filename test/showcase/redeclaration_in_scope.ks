
println("TEST SCOPE MODIFICATION");

let x = 10;

if true {
    x = 20;
    println(x);
}

println(x);