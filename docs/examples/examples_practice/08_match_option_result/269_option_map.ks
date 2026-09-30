// ==================================================================
// Exemple 269 — map sur un Option
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Transformer la valeur si elle existe, sinon rester None.
// ------------------------------------------------------------------
// Sortie attendue :
//   Some(20)
//   None
// ==================================================================

let a = Some(10);
let b: Option<int> = None;
println(a.map(x => x * 2));
println(b.map(x => x * 2));
