// ====================================================================
// Kastel — Remplacer du texte
// Notions : replace (première occurrence) et replace_all
// Résultat attendu :
//   le chat, un chien
//   le chat, le chien
// ====================================================================

let texte = "un chat, un chien";

println(texte.replace("un", "le"));
println(texte.replace_all("un", "le"));
