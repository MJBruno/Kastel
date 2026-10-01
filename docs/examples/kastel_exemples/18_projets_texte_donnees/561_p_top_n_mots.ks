// ==================================================================
// Exemple 561 — Les N mots les plus fréquents
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : compter puis extraire les maximums successifs.
// ------------------------------------------------------------------
// Sortie attendue :
//   a : 3
//   b : 2
// ==================================================================

let compte = dict();
for m in "a b a c b a".split(" ") {
    compte[m] = compte.get_or(m, 0) + 1;
}

for rang in range(2) {
    let meilleur = "";
    let max_n = 0;
    for m in compte {
        if compte[m] > max_n {
            max_n = compte[m];
            meilleur = m;
        }
    }
    println("{} : {}", meilleur, max_n);
    compte.remove(meilleur);
}
