let nom = "Alice";
let age = 30;

println("Hello {}", nom);                    // Hello Alice
println("Bonjour {}, tu as {} ans", nom, age); // Bonjour Alice, tu as 30 ans
println("Age: " + age);                       // Age: 30  (concaténation, fonctionne aussi)

let message = format("Score: {}/{}", 8, 10);  // "Score: 8/10", sans l'afficher
println(message);