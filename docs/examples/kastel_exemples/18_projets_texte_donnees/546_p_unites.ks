// ==================================================================
// Exemple 546 — Convertisseur d'unités
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : kilomètres en miles, kilos en livres.
// ------------------------------------------------------------------
// Sortie attendue :
//   6.21 miles
//   154.32 livres
// ==================================================================

func km_en_miles(km: float) -> float { return km * 0.621371; }
func kg_en_livres(kg: float) -> float { return kg * 2.20462; }

println("{:.2f} miles", km_en_miles(10.0));
println("{:.2f} livres", kg_en_livres(70.0));
