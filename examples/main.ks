// import math;

// const name = input("Comment tu t'appel?\n");

// let a = 25;

// if (a < 18) {
//     println("Mineur!!");
// } else {
//     println("Majeur!!");
// }

// let s = format("Bonjour {}, tu as {} ans", name, a);

// println(s);

// for ( i in [2, 6, 7, 4, 1, 9]) {
//     if (i % 2 == 0) {
//         println("Item: {}", i);
//     }
// }

let user = { name: "Bruno", age: 25 };

println(user.name);   // Bruno
println(user.age);    // 25

user.age = 26;
println(user.age);    // 26

user.city = "Antananarivo";   // nouveau champ, ajouté dynamiquement
println(user);                 // {name: Bruno, age: 26, city: Antananarivo}