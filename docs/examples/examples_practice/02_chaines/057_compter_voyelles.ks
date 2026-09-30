// ==================================================================
// Exemple 057 — Compter les voyelles
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// Parcourir la chaîne et tester chaque caractère.
// ------------------------------------------------------------------
// Sortie attendue :
//   5
// ==================================================================

let texte = "programmation";
let voyelles = "aeiou";
let n = 0;
for c in texte {
    if voyelles.contains(c) {
        n += 1;
    }
}
println(n);
