// ====================================================================
// Kastel — Canal borné
// Notions : channel(n), try_send, is_full, capacity
// Résultat attendu :
//   capacité = 2
//   true
//   true
//   false
//   true
// ====================================================================

let c = channel(2);

println("capacité = {}", c.capacity());
println(c.try_send(1));
println(c.try_send(2));
println(c.try_send(3));    // plein : refusé
println(c.is_full());
