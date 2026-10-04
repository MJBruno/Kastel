// ====================================================================
// Kastel — PGCD d'Euclide
// Notions : while, variable temporaire
// Résultat attendu :
//   pgcd(48, 18) = 6
//   pgcd(17, 5) = 1
// ====================================================================

func pgcd(a, b) {
    while b != 0 {
        let reste = a % b;
        a = b;
        b = reste;
    }
    return a;
}

println("pgcd(48, 18) = {}", pgcd(48, 18));
println("pgcd(17, 5) = {}", pgcd(17, 5));
