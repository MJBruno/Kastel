// ==================================================================
// Exemple 455 — Bulletin de notes
// Catégorie : Mini-projets
// ------------------------------------------------------------------
// Moyenne par étudiant et meilleur résultat.
// ------------------------------------------------------------------
// Sortie attendue :
//   Ada : 17.0
//   Alan : 13.0
//   Grace : 17.0
//   meilleur : Ada
// ==================================================================

let notes = {
    "Ada": [16, 18, 17],
    "Alan": [12, 14, 13],
    "Grace": [19, 15, 17]
};

let meilleur = "";
let meilleure_moyenne = 0.0;
for nom in notes {
    let somme = 0;
    for n in notes[nom] {
        somme += n;
    }
    let moyenne = somme / notes[nom].size();
    println("{} : {:.1f}", nom, moyenne);
    if moyenne > meilleure_moyenne {
        meilleure_moyenne = moyenne;
        meilleur = nom;
    }
}
println("meilleur : " + meilleur);
