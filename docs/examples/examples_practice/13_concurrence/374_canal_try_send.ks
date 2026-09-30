// ==================================================================
// Exemple 374 — try_send sur canal plein
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// try_send renvoie false quand le canal est plein ou fermé.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
// ==================================================================

let c = channel<int>(1);
println(c.try_send(1));
println(c.try_send(2));
