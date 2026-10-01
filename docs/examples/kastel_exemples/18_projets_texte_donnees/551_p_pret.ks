// ==================================================================
// Exemple 551 — Mensualité d'un prêt
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : formule des annuités constantes (10 000 € à 5 % sur 12 mois).
// ------------------------------------------------------------------
// Sortie attendue :
//   856.07
// ==================================================================

func mensualite(capital: float, taux_annuel: float, mois: int) -> float {
    let r = taux_annuel / 12.0;
    let puissance = 1.0;
    for i in range(mois) {
        puissance *= 1.0 + r;
    }
    return capital * r * puissance / (puissance - 1.0);
}

println("{:.2f}", mensualite(10000.0, 0.05, 12));
