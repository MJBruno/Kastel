// ==================================================================
// Exemple 554 — Histogramme en texte
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : une barre de # par valeur.
// ------------------------------------------------------------------
// Sortie attendue :
//   A | ###
//   B | #####
//   C | ##
// ==================================================================

let donnees = {"A": 3, "B": 5, "C": 2};
for cle in donnees {
    println(cle + " | " + "#".repeat(donnees[cle]));
}
