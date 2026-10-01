// ==================================================================
// Exemple 469 — Taureaux et vaches
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : comparer deux nombres chiffre par chiffre.
// ------------------------------------------------------------------
// Sortie attendue :
//   2 taureaux, 2 vaches
//   0 taureaux, 0 vaches
//   4 taureaux, 0 vaches
// ==================================================================

func comparer(secret: str, essai: str) {
    let taureaux = 0;
    let vaches = 0;
    for i in range(secret.size()) {
        if secret.char_at(i) == essai.char_at(i) {
            taureaux += 1;
        } else if secret.contains(essai.char_at(i)) {
            vaches += 1;
        }
    }
    println("{} taureaux, {} vaches", taureaux, vaches);
}

comparer("1234", "1243");
comparer("1234", "5678");
comparer("1234", "1234");
