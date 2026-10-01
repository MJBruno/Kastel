// ==================================================================
// Exemple 376 — Recevoir sur un canal fermé
// Catégorie : Concurrence (tâches, canaux, verrous)
// ------------------------------------------------------------------
// Après close et épuisement, recv lève ChannelClosed.
// ------------------------------------------------------------------
// Sortie attendue :
//   1
//   ChannelClosed
// ==================================================================

let c = channel<int>();
c.send(1);
c.close();
println(c.recv());
try {
    c.recv();
} catch (e: Err) {
    println(e.kind);
}
