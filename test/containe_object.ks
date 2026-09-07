println("TEST CONTAINS OBJETS");

let a = { name: "Alice" };
let b = { name: "Alice" };

let values = [a];

println(values.contains(a));
println(values.contains(b));