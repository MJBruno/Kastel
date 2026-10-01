// ==================================================================
// Exemple 631 — Deux pointeurs sur liste triée
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : trouver deux nombres de somme donnée sans double boucle.
// ------------------------------------------------------------------
// Sortie attendue :
//   4 + 11
// ==================================================================

let v = [1, 2, 4, 7, 11, 15];
let cible = 15;
let g = 0;
let d = v.size() - 1;

while g < d {
    let s = v[g] + v[d];
    if s == cible {
        println("{} + {}", v[g], v[d]);
        break;
    } else if s < cible {
        g += 1;
    } else {
        d -= 1;
    }
}
