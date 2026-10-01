// ==================================================================
// Exemple 508 — Automate cellulaire (règle 90)
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : chaque cellule devient le XOR de ses deux voisines ; on obtient un triangle de Sierpinski.
// ------------------------------------------------------------------
// Sortie attendue :
//   ....#....
//   ...#.#...
//   ..#...#..
//   .#.#.#.#.
// ==================================================================

let n = 9;
let ligne = [];
for i in range(n) { ligne.add(0); }
ligne[4] = 1;

func afficher(l: List<int>) {
    let s = "";
    for c in l { s += c == 1 ? "#" : "."; }
    println(s);
}

for gen in range(4) {
    afficher(ligne);
    let suivante = [];
    for i in range(n) {
        let g = i > 0 ? ligne[i - 1] : 0;
        let d = i < n - 1 ? ligne[i + 1] : 0;
        suivante.add(g ^ d);
    }
    ligne = suivante;
}
