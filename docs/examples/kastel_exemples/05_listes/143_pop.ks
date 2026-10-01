// ==================================================================
// Exemple 143 — Dépiler avec pop()
// Catégorie : Listes
// ------------------------------------------------------------------
// pop() retire et renvoie le dernier élément (None si la liste est vide).
// ------------------------------------------------------------------
// Sortie attendue :
//   3
//   2
//   [1]
//   None
// ==================================================================

let v = [1, 2, 3];
println(v.pop());
println(v.pop());
println(v);
let vide = [];
println(vide.pop());
