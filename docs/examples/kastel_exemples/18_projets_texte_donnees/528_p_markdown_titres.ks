// ==================================================================
// Exemple 528 — Table des matières Markdown
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : extraire les titres et leur niveau.
// ------------------------------------------------------------------
// Sortie attendue :
//   1 Titre
//   2 Sous-titre
//   3 Détail
// ==================================================================

let doc = "# Titre\ntexte\n## Sous-titre\nautre texte\n### Détail";

for ligne in doc.split("\n") {
    if ligne.starts_with("#") {
        let niveau = 0;
        while ligne.char_at(niveau) == "#" {
            niveau += 1;
        }
        println("{} {}", niveau, ligne.slice(niveau, ligne.size()).trim());
    }
}
