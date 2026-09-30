// ==================================================================
// Exemple 115 — Fonction fléchée avec un bloc
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// Avec des accolades, on écrit plusieurs instructions et un return.
// ------------------------------------------------------------------
// Sortie attendue :
//   positif
//   négatif
// ==================================================================

let classer = n => {
    if n >= 0 {
        return "positif";
    }
    return "négatif";
};
println(classer(5));
println(classer(-5));
