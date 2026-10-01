// ==================================================================
// Exemple 521 — Chiffres romains vers entier
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : parcourir de droite à gauche ; un symbole plus petit que son voisin se soustrait.
// ------------------------------------------------------------------
// Sortie attendue :
//   1994
//   42
// ==================================================================

let valeurs = {"I": 1, "V": 5, "X": 10, "L": 50, "C": 100, "D": 500, "M": 1000};

func vers_entier(s: str) -> int {
    let total = 0;
    let precedent = 0;
    for i in range(s.size() - 1, -1, -1) {
        let v = valeurs[s.char_at(i)];
        if v < precedent {
            total -= v;
        } else {
            total += v;
        }
        precedent = v;
    }
    return total;
}

println(vers_entier("MCMXCIV"));
println(vers_entier("XLII"));
