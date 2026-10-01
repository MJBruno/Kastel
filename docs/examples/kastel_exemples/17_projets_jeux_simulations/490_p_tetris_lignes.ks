// ==================================================================
// Exemple 490 — Tétris : effacer les lignes complètes
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : retirer les lignes pleines d'une grille et compter les points.
// ------------------------------------------------------------------
// Sortie attendue :
//   lignes effacées : 2
//   [0, 0, 0, 0]
//   [0, 0, 0, 0]
//   [0, 1, 0, 0]
//   [1, 0, 1, 1]
// ==================================================================

let grille = [
    [1, 1, 1, 1],
    [0, 1, 0, 0],
    [1, 1, 1, 1],
    [1, 0, 1, 1]
];

let restantes = [];
let effacees = 0;
for ligne in grille {
    let pleine = true;
    for c in ligne { if c == 0 { pleine = false; } }
    if pleine { effacees += 1; } else { restantes.add(ligne); }
}
// On complète en haut avec des lignes vides.
while restantes.size() < grille.size() {
    restantes.insert(0, [0, 0, 0, 0]);
}
println("lignes effacées : " + str(effacees));
for l in restantes { println(l); }
