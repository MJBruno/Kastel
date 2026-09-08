println("TEST TABLEAU OBJETS");

let users = [
    { name: "Alice", age: 20 },
    { name: "Bob", age: 30 },
    { name: "Charlie", age: 40 }
];

println(users[0].name);
println(users[1].age);
println(users[2].name);

users[1].age = 31;

println(users[1].age);