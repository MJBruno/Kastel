// ==================================================================
// Exemple 514 — Compteur de mots
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : total de mots, mots différents et mot le plus fréquent.
// ------------------------------------------------------------------
// Sortie attendue :
//   6 mots, 3 différents
//   plus fréquent : un (3)
// ==================================================================

let texte = "un deux un trois deux un";
let compte = dict();
for m in texte.split(" ") {
    compte[m] = compte.get_or(m, 0) + 1;
}

let meilleur = "";
let max_n = 0;
for m in compte {
    if compte[m] > max_n {
        max_n = compte[m];
        meilleur = m;
    }
}
println("{} mots, {} différents", texte.split(" ").size(), compte.size());
println("plus fréquent : {} ({})", meilleur, max_n);
