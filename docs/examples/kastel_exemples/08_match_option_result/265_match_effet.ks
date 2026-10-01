// ==================================================================
// Exemple 265 — match comme instruction
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Les bras peuvent contenir n'importe quelles instructions.
// ------------------------------------------------------------------
// Sortie attendue :
//   133
// ==================================================================

let total = 0;
for n in range(1, 8) {
    match n {
        1 | 2 | 3 => { total += 1; }
        4..=6 => { total += 10; }
        _ => { total += 100; }
    }
}
println(total);
