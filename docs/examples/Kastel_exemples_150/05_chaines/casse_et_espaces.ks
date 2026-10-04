// ====================================================================
// Kastel — Casse et espaces
// Notions : upper, lower, trim, trim_start, trim_end
// Résultat attendu :
//   [Bonjour Kastel]
//   BONJOUR KASTEL
//   bonjour kastel
//   [Bonjour Kastel  ]
//   [  Bonjour Kastel]
// ====================================================================

let texte = "  Bonjour Kastel  ";

println("[{}]", texte.trim());
println(texte.trim().upper());
println(texte.trim().lower());
println("[{}]", texte.trim_start());
println("[{}]", texte.trim_end());
