// ==================================================================
// Exemple 040 — Supprimer les espaces avec trim()
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// trim() enlève les espaces aux deux bouts, trim_start()/trim_end() d'un seul côté.
// ------------------------------------------------------------------
// Sortie attendue :
//   [bonjour]
//   [bonjour   ]
//   [   bonjour]
// ==================================================================

let s = "   bonjour   ";
println("[" + s.trim() + "]");
println("[" + s.trim_start() + "]");
println("[" + s.trim_end() + "]");
