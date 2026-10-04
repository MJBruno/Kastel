// ====================================================================
// Kastel — Sémaphore
// Notions : semaphore(n), try_acquire, release, available
// Résultat attendu :
//   places : 2
//   true
//   true
//   false
//   places libres : 0
//   places libres : 1
// ====================================================================

let s = semaphore(2);
println("places : {}", s.capacity());

println(s.try_acquire());
println(s.try_acquire());
println(s.try_acquire());    // plus de place

println("places libres : {}", s.available());
s.release();
println("places libres : {}", s.available());
