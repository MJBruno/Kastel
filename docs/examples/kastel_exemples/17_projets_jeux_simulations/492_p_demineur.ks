// ==================================================================
// Exemple 492 — Démineur : compter les mines voisines
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : afficher pour chaque case le nombre de mines autour (* = mine).
// ------------------------------------------------------------------
// Sortie attendue :
//   *21
//   12*
//   011
// ==================================================================

let mines = [
    ["*", ".", "."],
    [".", ".", "*"],
    [".", ".", "."]
];

for i in range(3) {
    let ligne = "";
    for j in range(3) {
        if mines[i][j] == "*" {
            ligne += "*";
            continue;
        }
        let n = 0;
        for di in range(-1, 2) {
            for dj in range(-1, 2) {
                let a = i + di;
                let b = j + dj;
                if a >= 0 && a < 3 && b >= 0 && b < 3 && mines[a][b] == "*" { n += 1; }
            }
        }
        ligne += str(n);
    }
    println(ligne);
}
