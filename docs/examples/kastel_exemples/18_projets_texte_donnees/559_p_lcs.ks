// ==================================================================
// Exemple 559 — Plus longue sous-séquence commune
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : programmation dynamique sur deux chaînes.
// ------------------------------------------------------------------
// Sortie attendue :
//   4
// ==================================================================

func lcs(a: str, b: str) -> int {
    let n = a.size();
    let m = b.size();
    let t = [];
    for i in range(n + 1) {
        let ligne = [];
        for j in range(m + 1) { ligne.add(0); }
        t.add(ligne);
    }
    for i in range(1, n + 1) {
        for j in range(1, m + 1) {
            if a.char_at(i - 1) == b.char_at(j - 1) {
                t[i][j] = t[i - 1][j - 1] + 1;
            } else {
                t[i][j] = max(t[i - 1][j], t[i][j - 1]);
            }
        }
    }
    return t[n][m];
}

println(lcs("ABCBDAB", "BDCABA"));
