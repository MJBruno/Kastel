// ==================================================================
// Exemple 414 — Décimal vers binaire (à la main)
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Diviser par 2 en gardant les restes.
// ------------------------------------------------------------------
// Sortie attendue :
//   1010
//   11111111
// ==================================================================

func en_binaire(n: int) -> str {
    if n == 0 {
        return "0";
    }
    let s = "";
    while n > 0 {
        s = str(n % 2) + s;
        n = idiv(n, 2);
    }
    return s;
}

println(en_binaire(10));
println(en_binaire(255));
