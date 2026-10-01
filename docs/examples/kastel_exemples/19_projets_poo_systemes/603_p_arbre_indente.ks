// ==================================================================
// Exemple 603 — Afficher une arborescence
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : indenter chaque nom selon sa profondeur.
// ------------------------------------------------------------------
// Sortie attendue :
//   racine
//     src
//       main.ks
//     docs
//       guide.md
//       api.md
// ==================================================================

let arbre = [(0, "racine"), (1, "src"), (2, "main.ks"), (1, "docs"), (2, "guide.md"), (2, "api.md")];

for n in arbre {
    println("  ".repeat(n[0]) + n[1]);
}
