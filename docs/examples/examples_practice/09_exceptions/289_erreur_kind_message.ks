// ==================================================================
// Exemple 289 — kind, message et to_string()
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Une erreur runtime expose son genre et son message.
// ------------------------------------------------------------------
// Sortie attendue :
//   ArrayIndexOutOfBounds
//   true
// ==================================================================

try {
    let v = [1, 2, 3];
    println(v[9]);
} catch (e: Err) {
    println(e.kind);
    println(e.message.size() > 0);
}
