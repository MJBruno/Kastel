// ==================================================================
// Exemple 612 — Registre de plugins
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : associer un nom à une fonction et l'appeler dynamiquement.
// ------------------------------------------------------------------
// Sortie attendue :
//   42
//   49
//   plugin inconnu : cube
// ==================================================================

let registre = dict();
registre["double"] = x => x * 2;
registre["carre"] = x => x * x;

func appeler(nom: str, valeur: int) {
    if !registre.contains(nom) {
        println("plugin inconnu : " + nom);
        return;
    }
    let f = registre[nom];
    println(f(valeur));
}

appeler("double", 21);
appeler("carre", 7);
appeler("cube", 3);
