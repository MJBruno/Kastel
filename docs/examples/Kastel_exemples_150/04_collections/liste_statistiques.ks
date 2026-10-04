// ====================================================================
// Kastel — Statistiques simples
// Notions : minimum, maximum, moyenne
// Résultat attendu :
//   min = 4
//   max = 42
//   moyenne = 18.0
// ====================================================================

let valeurs = [4, 8, 15, 16, 23, 42];

let plus_petit = valeurs[0];
let plus_grand = valeurs[0];
let somme = 0;

for v in valeurs {
    if v < plus_petit {
        plus_petit = v;
    }
    if v > plus_grand {
        plus_grand = v;
    }
    somme += v;
}

println("min = {}", plus_petit);
println("max = {}", plus_grand);
println("moyenne = {:.1f}", somme / valeurs.size());
