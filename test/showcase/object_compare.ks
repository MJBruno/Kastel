println("TEST EGALITE OBJETS");

let a = { name: "Alice", age: 20 };
let b = { name: "Alice", age: 20 };
let c = a;

println(a == b);  // Deux objet different => false
println(a == c);  // Meme reference => true