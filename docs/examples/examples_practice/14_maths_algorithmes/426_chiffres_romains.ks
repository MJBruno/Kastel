// ==================================================================
// Exemple 426 — Nombre vers chiffres romains
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Soustractions gloutonnes sur une table de valeurs.
// ------------------------------------------------------------------
// Sortie attendue :
//   MCMXCIV
//   MMXXVI
// ==================================================================

func romain(n: int) -> str {
    let valeurs = [1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1];
    let lettres = ["M", "CM", "D", "CD", "C", "XC", "L", "XL", "X", "IX", "V", "IV", "I"];
    let s = "";
    for i in range(valeurs.size()) {
        while n >= valeurs[i] {
            s += lettres[i];
            n -= valeurs[i];
        }
    }
    return s;
}

println(romain(1994));
println(romain(2026));
