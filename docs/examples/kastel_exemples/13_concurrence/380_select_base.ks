// ==================================================================
// Exemple 380 — select : attendre plusieurs canaux
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// select([c1, c2]) renvoie [numéro du canal, valeur, canal fermé ?].
// ------------------------------------------------------------------
// Sortie attendue :
//   1
//   99
// ==================================================================

let a = channel<int>();
let b = channel<int>();
b.send(99);

let r = select([a, b]);
println(r[0]);   // 1 : c'est le canal b qui était prêt
println(r[1]);   // 99
