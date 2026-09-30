// ==================================================================
// Exemple 138 — Index hors limites
// Catégorie : Listes
// ------------------------------------------------------------------
// Lire au-delà de la fin lève ArrayIndexOutOfBounds.
// ------------------------------------------------------------------
// Sortie attendue :
//   ArrayIndexOutOfBounds
// ==================================================================

let v = [1, 2, 3];
try {
    println(v[10]);
} catch (e: Err) {
    println(e.kind);
}
