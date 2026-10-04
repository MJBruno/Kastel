// ====================================================================
// Kastel — Réception non bloquante
// Notions : try_recv renvoie None si le canal est vide
// Résultat attendu :
//   true
//   7
// ====================================================================

let c = channel();

let vide = c.try_recv();
println(vide.is_none());

c.send(7);
println(c.try_recv().unwrap());
