// ==================================================================
// Exemple 375 — Fermer un canal
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// close() interdit les nouveaux envois ; is_closed() le signale.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
// ==================================================================

let c = channel<int>(1);
c.close();
println(c.is_closed());
println(c.try_send(1));
