// ==================================================================
// Exemple 596 — Décorateur : compter les appels
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : envelopper une fonction pour mesurer son usage.
// ------------------------------------------------------------------
// Sortie attendue :
//   16
//   3
// ==================================================================

func avec_compteur(f, compteur: List<int>) {
    return x => {
        compteur[0] += 1;
        return f(x);
    };
}

let appels = [0];
let carre = avec_compteur(x => x * x, appels);
carre(2);
carre(3);
println(carre(4));
println(appels[0]);
