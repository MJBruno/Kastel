// ==================================================================
// Exemple 381 — select avec délai
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// select([canaux], ms) abandonne après ms millisecondes : [-1, None, false].
// ------------------------------------------------------------------
// Sortie attendue :
//   -1
//   None
// ==================================================================

let c = channel<int>();
let r = select([c], 10);
println(r[0]);
println(r[1]);
