// ==================================================================
// Exemple 498 — Bowling : calcul du score
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : strikes, spares et bonus sur 10 frames.
// ------------------------------------------------------------------
// Sortie attendue :
//   300
//   0
//   150
// ==================================================================

func score(lancers: List<int>) -> int {
    let total = 0;
    let i = 0;
    for frame in range(10) {
        if lancers[i] == 10 {
            total += 10 + lancers[i + 1] + lancers[i + 2];
            i += 1;
        } else if lancers[i] + lancers[i + 1] == 10 {
            total += 10 + lancers[i + 2];
            i += 2;
        } else {
            total += lancers[i] + lancers[i + 1];
            i += 2;
        }
    }
    return total;
}

println(score([10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10]));   // partie parfaite
println(score([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]));
println(score([5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5]));
