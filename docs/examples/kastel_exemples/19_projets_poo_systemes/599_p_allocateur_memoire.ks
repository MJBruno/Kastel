// ==================================================================
// Exemple 599 — Allocateur mémoire (first fit)
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : chaque demande prend le premier bloc assez grand.
// ------------------------------------------------------------------
// Sortie attendue :
//   212 -> bloc 1
//   417 -> bloc 4
//   112 -> bloc 1
//   426 -> échec
// ==================================================================

let blocs = [100, 500, 200, 300, 600];

for demande in [212, 417, 112, 426] {
    let choisi = -1;
    for i in range(blocs.size()) {
        if blocs[i] >= demande {
            choisi = i;
            break;
        }
    }
    if choisi == -1 {
        println("{} -> échec", demande);
    } else {
        blocs[choisi] = blocs[choisi] - demande;
        println("{} -> bloc {}", demande, choisi);
    }
}
