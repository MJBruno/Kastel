// ====================================================================
// Kastel — Débordement d'entier
// Notions : les entiers 64 bits lèvent une erreur au lieu de boucler
// Résultat attendu :
//   Débordement détecté
// ====================================================================

try {
    let grand = 9223372036854775807;
    let boom = grand + 1;
    println("jamais affiché : {}", boom);
} catch (e: Err) {
    println("Débordement détecté");
}
