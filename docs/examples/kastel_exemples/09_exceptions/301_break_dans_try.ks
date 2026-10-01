// ==================================================================
// Exemple 301 — break et continue dans try
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Le finally s'exécute à chaque sortie du try, même par break ou continue.
// ------------------------------------------------------------------
// Sortie attendue :
//   4
//   4
// ==================================================================

let passages = 0;
let i = 0;
while i < 5 {
    i += 1;
    try {
        if i == 2 { continue; }
        if i == 4 { break; }
    } finally {
        passages += 1;
    }
}
println(i);
println(passages);
