// ==================================================================
// Exemple 373 — try_recv : sans attendre
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// Renvoie un Option : Some(valeur) ou None si rien n'est disponible.
// ------------------------------------------------------------------
// Sortie attendue :
//   None
//   Some(7)
//   None
// ==================================================================

let c = channel<int>();
println(c.try_recv());
c.send(7);
println(c.try_recv());
println(c.try_recv());
