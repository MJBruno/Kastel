// ==================================================================
// Exemple 558 — Distance de Levenshtein
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : nombre minimal d'insertions, suppressions et substitutions entre deux mots.
// ------------------------------------------------------------------
// Sortie attendue :
//   3
//   2
//   3
// ==================================================================

func distance(a: str, b: str) -> int {
    let n = a.size();
    let m = b.size();
    let d = [];
    for i in range(n + 1) {
        let ligne = [];
        for j in range(m + 1) { ligne.add(0); }
        d.add(ligne);
    }
    for i in range(n + 1) { d[i][0] = i; }
    for j in range(m + 1) { d[0][j] = j; }

    for i in range(1, n + 1) {
        for j in range(1, m + 1) {
            let cout = a.char_at(i - 1) == b.char_at(j - 1) ? 0 : 1;
            d[i][j] = min(min(d[i - 1][j] + 1, d[i][j - 1] + 1), d[i - 1][j - 1] + cout);
        }
    }
    return d[n][m];
}

println(distance("kitten", "sitting"));
println(distance("flaw", "lawn"));
println(distance("", "abc"));
