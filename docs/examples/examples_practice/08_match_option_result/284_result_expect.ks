// ==================================================================
// Exemple 284 — expect : extraire avec un message
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Comme unwrap, mais l'erreur porte votre message.
// ------------------------------------------------------------------
// Sortie attendue :
//   ResultUnwrap
// ==================================================================

let r: Result<int, str> = Err("disque plein");
try {
    r.expect("écriture impossible");
} catch (e: Err) {
    println(e.kind);
}
