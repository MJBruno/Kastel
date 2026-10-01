// ==================================================================
// Exemple 478 — Mélange de cartes (Fisher-Yates)
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : mélanger un paquet et vérifier qu'aucune carte n'a disparu.
// ------------------------------------------------------------------
// Sortie attendue :
//   [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
//   10
// ==================================================================

let graine = 42;

// Générateur pseudo-aléatoire déterministe (congruentiel linéaire).
func alea(n: int) -> int {
    graine = (graine * 1103515245 + 12345) % 2147483648;
    return idiv(graine, 65536) % n;
}

       let paquet = [];
       for i in range(1, 11) { paquet.add(i); }

       // Fisher-Yates : on échange chaque case avec une case aléatoire précédente.
       let i = paquet.size() - 1;
       while i > 0 {
           let j = alea(i + 1);
           let tmp = paquet[i];
           paquet[i] = paquet[j];
           paquet[j] = tmp;
           i -= 1;
       }

       let copie = paquet.copy();
       copie.sort();
       println(copie);
       println(paquet.size());
