// ====================================================================
// Kastel — Opérateurs bit à bit
// Notions : & | ^ ~ << >> (parenthéser : ils lient moins fort que ==)
// Résultat attendu :
//   6 & 3 = 2
//   6 | 3 = 7
//   6 ^ 3 = 5
//   ~6 = -7
//   1 << 4 = 16
//   256 >> 2 = 64
// ====================================================================

println("6 & 3 = {}", 6 & 3);
println("6 | 3 = {}", 6 | 3);
println("6 ^ 3 = {}", 6 ^ 3);
println("~6 = {}", ~6);
println("1 << 4 = {}", 1 << 4);
println("256 >> 2 = {}", 256 >> 2);

// Attention : en Kastel, & | ^ ont une priorité plus basse que ==.
// On parenthèse donc toujours les expressions bit à bit :
let est_pair = (42 & 1) == 0;
println("42 est pair : {}", est_pair);
