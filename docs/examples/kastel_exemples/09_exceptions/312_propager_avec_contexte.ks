// ==================================================================
// Exemple 312 — Ajouter du contexte puis relancer
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Attraper, enrichir le message, relancer.
// ------------------------------------------------------------------
// Sortie attendue :
//   démarrage impossible : fichier absent
// ==================================================================

func lire_config() {
    throw "fichier absent";
}

func demarrer() {
    try {
        lire_config();
    } catch (e) {
        throw "démarrage impossible : " + e;
    }
}

try {
    demarrer();
} catch (e) {
    println(e);
}
