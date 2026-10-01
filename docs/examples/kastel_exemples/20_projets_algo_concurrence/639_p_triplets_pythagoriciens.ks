// ==================================================================
// Exemple 639 — Triplets pythagoriciens
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : a² + b² = c² avec c ≤ 30.
// ------------------------------------------------------------------
// Sortie attendue :
//   11
// ==================================================================

let n = 0;
for c in range(1, 31) {
    for b in range(1, c) {
        for a in range(1, b) {
            if a * a + b * b == c * c {
                n += 1;
            }
        }
    }
}
println(n);
