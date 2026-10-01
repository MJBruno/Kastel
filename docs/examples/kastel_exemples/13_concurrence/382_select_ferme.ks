// ==================================================================
// Exemple 382 — select et canal fermé
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// Un canal fermé est signalé par le troisième élément.
// ------------------------------------------------------------------
// Sortie attendue :
//   1
//   true
// ==================================================================

let a = channel<int>();
let b = channel<int>();
b.close();

let r = select([a, b]);
println(r[0]);
println(r[2]);
