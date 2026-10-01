// ==================================================================
// Exemple 547 — Convertisseur de devises
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : une table de taux par rapport à l'euro.
// ------------------------------------------------------------------
// Sortie attendue :
//   110.00
//   85.00
//   100.00
// ==================================================================

let taux = {"EUR": 1.0, "USD": 1.1, "GBP": 0.85};

func convertir(montant: float, de: str, vers: str) -> float {
    return montant / taux[de] * taux[vers];
}

println("{:.2f}", convertir(100.0, "EUR", "USD"));
println("{:.2f}", convertir(100.0, "EUR", "GBP"));
println("{:.2f}", convertir(110.0, "USD", "EUR"));
