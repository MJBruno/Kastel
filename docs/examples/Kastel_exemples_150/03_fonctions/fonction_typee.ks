// ====================================================================
// Kastel — Fonction typée
// Notions : paramètres typés et type de retour ->
// Résultat attendu :
//   13.5
//   Aire : 13.50
// ====================================================================

func aire_rectangle(largeur: float, hauteur: float) -> float {
    return largeur * hauteur;
}

println(aire_rectangle(3.0, 4.5));
println("Aire : {:.2f}", aire_rectangle(3.0, 4.5));
