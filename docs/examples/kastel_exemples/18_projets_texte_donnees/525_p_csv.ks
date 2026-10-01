// ==================================================================
// Exemple 525 — Lecteur CSV
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : lire un texte CSV avec en-tête et calculer une moyenne.
// ------------------------------------------------------------------
// Sortie attendue :
//   2 lignes, âge moyen 38.5
// ==================================================================

let donnees = "nom,age\nAda,36\nAlan,41";
let lignes = donnees.split("\n");
let entetes = lignes[0].split(",");

let lignes_dict = [];
for i in range(1, lignes.size()) {
    let champs = lignes[i].split(",");
    let ligne = dict();
    for j in range(entetes.size()) {
        ligne[entetes[j]] = champs[j];
    }
    lignes_dict.add(ligne);
}

let somme = 0;
for l in lignes_dict {
    somme += l["age"].to_int();
}
println("{} lignes, âge moyen {}", lignes_dict.size(), somme / lignes_dict.size());
