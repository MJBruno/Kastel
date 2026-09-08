println("TEST FOR IN + MUTATION");

let users = [
    { name: "Alice", age: 20 },
    { name: "Bob", age: 30 },
    { name: "Charlie", age: 40 }
];

for user in users {
    user.age += 1;
}

for user in users {
    println(user.age);
}