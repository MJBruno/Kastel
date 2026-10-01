// ==================================================================
// Exemple 479 — Jeu de la vie : un planeur
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : faire avancer un planeur sur une grille 6 x 6 pendant 4 générations et compter les cellules vivantes (toujours 5).
// ------------------------------------------------------------------
// Sortie attendue :
//   5
// ==================================================================

let N = 6;

func vide() -> List<List<int>> {
    let g = [];
    for i in range(N) {
        let ligne = [];
        for j in range(N) { ligne.add(0); }
        g.add(ligne);
    }
    return g;
}

func voisins(g: List<List<int>>, i: int, j: int) -> int {
    let n = 0;
    for di in range(-1, 2) {
        for dj in range(-1, 2) {
            if di == 0 && dj == 0 { continue; }
            let a = i + di;
            let b = j + dj;
            if a >= 0 && a < N && b >= 0 && b < N { n += g[a][b]; }
        }
    }
    return n;
}

func etape(g: List<List<int>>) -> List<List<int>> {
    let h = vide();
    for i in range(N) {
        for j in range(N) {
            let v = voisins(g, i, j);
            h[i][j] = (g[i][j] == 1 && (v == 2 || v == 3)) || (g[i][j] == 0 && v == 3) ? 1 : 0;
        }
    }
    return h;
}

func vivantes(g: List<List<int>>) -> int {
    let n = 0;
    for l in g { for c in l { n += c; } }
    return n;
}

let g = vide();
g[0][1] = 1; g[1][2] = 1; g[2][0] = 1; g[2][1] = 1; g[2][2] = 1;
for gen in range(4) {
    g = etape(g);
}
println(vivantes(g));
