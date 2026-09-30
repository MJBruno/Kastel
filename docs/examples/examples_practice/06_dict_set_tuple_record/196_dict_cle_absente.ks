// ==================================================================
// Exemple 196 — Clé absente : erreur
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// Lire une clé qui n'existe pas lève une erreur qu'on peut attraper.
// ------------------------------------------------------------------
// Sortie attendue :
//   ObjectFieldNotFound
// ==================================================================

let d = {"a": 1};
try {
    println(d["zzz"]);
} catch (e: Err) {
    println(e.kind);
}
