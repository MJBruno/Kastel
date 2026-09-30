// ==================================================================
// Exemple 415 — Binaire vers décimal
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Multiplier par 2 et ajouter chaque bit.
// ------------------------------------------------------------------
// Sortie attendue :
//   10
//   255
// ==================================================================

func depuis_binaire(s: str) -> int {
    let n = 0;
    for c in s {
        n = n * 2 + int(c);
    }
    return n;
}

println(depuis_binaire("1010"));
println(depuis_binaire("11111111"));
