// ==================================================================
// Exemple 495 — Course de tortues
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : chaque tortue avance d'un pas aléatoire à chaque tour ; on vérifie que la course se termine.
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

       let positions = [0, 0, 0];
       let tours = 0;
       while positions.all(p => p < 20) {
           for i in range(3) {
               positions[i] += alea(3);
           }
           tours += 1;
       }
       println(tours > 0);
       println(positions.any(p => p >= 20));
