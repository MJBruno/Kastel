// ==================================================================
// Exemple 467 — Le pendu
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : afficher le mot avec les lettres trouvées et compter les erreurs.
// ------------------------------------------------------------------
// Sortie attendue :
//   _a____  erreurs=0
//   _a____  erreurs=1
//   _a_t__  erreurs=1
//   _a_te_  erreurs=1
//   _a_te_  erreurs=2
//   ka_te_  erreurs=2
//   kaste_  erreurs=2
//   kastel  erreurs=2
// ==================================================================

let mot = "kastel";
let essais = ["a", "z", "t", "e", "x", "k", "s", "l"];
let trouvees = Set();
let erreurs = 0;

func affichage() -> str {
    let s = "";
    for c in mot {
        s += trouvees.contains(c) ? c : "_";
    }
    return s;
}

for lettre in essais {
    if mot.contains(lettre) {
        trouvees.add(lettre);
    } else {
        erreurs += 1;
    }
    println(affichage() + "  erreurs=" + str(erreurs));
}
