// ==================================================================
// Exemple 081 — Sortir d'une boucle avec break
// Catégorie : Structures de contrôle
// ------------------------------------------------------------------
// break arrête immédiatement la boucle.
// ------------------------------------------------------------------
// Sortie attendue :
//   premier carré > 50 : 64
// ==================================================================

for i in range(1, 100) {
    if i * i > 50 {
        println("premier carré > 50 : " + str(i * i));
        break;
    }
}
