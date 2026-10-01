// ==================================================================
// Exemple 477 — Dé pseudo-aléatoire
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : lancer un dé avec un générateur maison, et vérifier que toutes les faces restent dans [1, 6].
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   true
// ==================================================================

let graine = 42;

// Générateur pseudo-aléatoire déterministe (congruentiel linéaire).
func alea(n: int) -> int {
    graine = (graine * 1103515245 + 12345) % 2147483648;
    return idiv(graine, 65536) % n;
}

       let ok = true;
       let vues = Set();
       for i in range(200) {
           let d = alea(6) + 1;
           vues.add(d);
           if d < 1 || d > 6 { ok = false; }
       }
       println(ok);
       println(vues.size() <= 6);   // au plus les 6 faces d'un dé
