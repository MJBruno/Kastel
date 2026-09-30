// ==================================================================
// Exemple 251 — Enum dans une liste
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// Compter les éléments d'une valeur donnée.
// ------------------------------------------------------------------
// Sortie attendue :
//   3
// ==================================================================

enum Etat { Actif, Inactif }

let etats = [Etat.Actif, Etat.Inactif, Etat.Actif, Etat.Actif];
let actifs = 0;
for e in etats {
    if e == Etat.Actif {
        actifs += 1;
    }
}
println(actifs);
