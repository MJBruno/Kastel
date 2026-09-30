// ==================================================================
// Exemple 268 — unwrap sur None
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Extraire un None lève OptionUnwrap.
// ------------------------------------------------------------------
// Sortie attendue :
//   OptionUnwrap
// ==================================================================

let rien: Option<int> = None;
try {
    rien.unwrap();
} catch (e: Err) {
    println(e.kind);
}
