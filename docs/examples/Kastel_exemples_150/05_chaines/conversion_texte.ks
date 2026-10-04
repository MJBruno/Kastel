// ====================================================================
// Kastel — Convertir du texte
// Notions : to_int, to_float, erreur de conversion
// Résultat attendu :
//   42
//   2.5
//   Conversion impossible
// ====================================================================

println("  42 ".to_int());
println("2.5".to_float());

try {
    let n = "abc".to_int();
    println(n);
} catch (e: Err) {
    println("Conversion impossible");
}
