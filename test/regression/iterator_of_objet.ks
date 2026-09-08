println("TEST FOR IN + OBJETS");

let users = [
    { name: "Alice", age: 20 },
    { name: "Bob", age: 30 },
    { name: "Charlie", age: 40 }
];

for user in users {
    println(user.name);
    println(user.age);
}