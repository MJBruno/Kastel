// ==================================================================
// Exemple 562 — Comparer deux versions d'un texte
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : lignes supprimées (-) et ajoutées (+).
// ------------------------------------------------------------------
// Sortie attendue :
//   - b
//   + d
// ==================================================================

let ancien = ["a", "b", "c"];
let nouveau = ["a", "c", "d"];

for l in ancien {
    if !nouveau.contains(l) {
        println("- " + l);
    }
}
for l in nouveau {
    if !ancien.contains(l) {
        println("+ " + l);
    }
}
