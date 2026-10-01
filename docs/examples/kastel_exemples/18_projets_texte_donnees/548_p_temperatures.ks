// ==================================================================
// Exemple 548 — Table de conversion des températures
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : Celsius vers Fahrenheit sur quelques valeurs.
// ------------------------------------------------------------------
// Sortie attendue :
//   0 C = 32.0 F
//   25 C = 77.0 F
//   100 C = 212.0 F
// ==================================================================

for c in [0, 25, 100] {
    let f = c * 9 / 5 + 32;
    println("{} C = {:.1f} F", c, f);
}
