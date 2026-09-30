// ==================================================================
// Exemple 428 — Changer de base
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Écrire un entier en base 2 à 16 avec une table de chiffres.
// ------------------------------------------------------------------
// Sortie attendue :
//   FF
//   11111111
//   377
// ==================================================================

func en_base(n: int, base: int) -> str {
    let chiffres = "0123456789ABCDEF";
    if n == 0 {
        return "0";
    }
    let s = "";
    while n > 0 {
        s = chiffres.char_at(n % base) + s;
        n = idiv(n, base);
    }
    return s;
}

println(en_base(255, 16));
println(en_base(255, 2));
println(en_base(255, 8));
