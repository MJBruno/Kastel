// ====================================================================
// Kastel — Canaux
// Notions : channel(), send, recv, size, is_empty (FIFO)
// Résultat attendu :
//   2
//   10
//   20
//   true
// ====================================================================

let c = channel();

c.send(10);
c.send(20);
println(c.size());

println(c.recv());
println(c.recv());
println(c.is_empty());
