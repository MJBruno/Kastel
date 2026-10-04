// ====================================================================
// Kastel — Compter les voyelles
// Notions : for sur une chaîne, contains
// Résultat attendu :
//   5
// ====================================================================

let compte = 0;

for c in "programmation" {
    if "aeiou".contains(c) {
        compte += 1;
    }
}

println(compte);
