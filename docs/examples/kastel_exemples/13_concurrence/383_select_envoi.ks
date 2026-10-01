// ==================================================================
// Exemple 383 — select en émission
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// Un couple (canal, valeur) demande un envoi.
// ------------------------------------------------------------------
// Sortie attendue :
//   0
//   5
// ==================================================================

let c = channel<int>(1);
let r = select([(c, 5)]);
println(r[0]);
println(c.recv());
