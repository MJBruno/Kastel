// ==================================================================
// Exemple 288 — try / catch
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Le bloc catch s'exécute quand le try lève une erreur.
// ------------------------------------------------------------------
// Sortie attendue :
//   erreur attrapée : DivisionByZero
//   le programme continue
// ==================================================================

try {
    let x = 10 / 0;
    println(x);
} catch (e: Err) {
    println("erreur attrapée : " + e.kind);
}
println("le programme continue");
