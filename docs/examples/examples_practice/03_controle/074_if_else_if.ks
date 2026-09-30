// ==================================================================
// Exemple 074 — Chaîne else if
// Catégorie : Structures de contrôle
// ------------------------------------------------------------------
// Plusieurs cas exclusifs testés dans l'ordre.
// ------------------------------------------------------------------
// Sortie attendue :
//   bien
// ==================================================================

let note = 14;
if note >= 16 {
    println("très bien");
} else if note >= 12 {
    println("bien");
} else if note >= 10 {
    println("passable");
} else {
    println("insuffisant");
}
