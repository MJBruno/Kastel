// ==================================================================
// Exemple 396 — reset d'un event
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// reset() le remet à l'état « non signalé ».
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
// ==================================================================

let e = event();
e.set();
println(e.is_set());
e.reset();
println(e.is_set());
