// ==================================================================
// Exemple 429 — Jeu de la vie : une étape
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Un oscillateur (le « clignotant ») de 3 cellules.
// ------------------------------------------------------------------
// Sortie attendue :
//   [0, 1, 0]
//   [0, 1, 0]
//   [0, 1, 0]
// ==================================================================

let g = [
    [0, 0, 0],
    [1, 1, 1],
    [0, 0, 0]
];

func voisins(g, i, j) -> int {
    let n = 0;
    for di in range(-1, 2) {
        for dj in range(-1, 2) {
            if di == 0 && dj == 0 { continue; }
            let a = i + di;
            let b = j + dj;
            if a >= 0 && a < 3 && b >= 0 && b < 3 {
                n += g[a][b];
            }
        }
    }
    return n;
}

let h = [[0, 0, 0], [0, 0, 0], [0, 0, 0]];
for i in range(3) {
    for j in range(3) {
        let v = voisins(g, i, j);
        if g[i][j] == 1 {
            h[i][j] = (v == 2 || v == 3) ? 1 : 0;
        } else {
            h[i][j] = v == 3 ? 1 : 0;
        }
    }
}
for ligne in h {
    println(ligne);
}
