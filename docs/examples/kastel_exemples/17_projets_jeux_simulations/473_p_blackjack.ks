// ==================================================================
// Exemple 473 — Blackjack : valeur d'une main
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : les figures valent 10 et l'as vaut 11 ou 1 pour ne pas dépasser 21.
// ------------------------------------------------------------------
// Sortie attendue :
//   21
//   21
//   24
// ==================================================================

func valeur(main: List<str>) -> int {
    let total = 0;
    let as_ = 0;
    for c in main {
        if c == "A" {
            total += 11;
            as_ += 1;
        } else if c == "K" || c == "Q" || c == "J" {
            total += 10;
        } else {
            total += int(c);
        }
    }
    while total > 21 && as_ > 0 {
        total -= 10;
        as_ -= 1;
    }
    return total;
}

println(valeur(["A", "K"]));
println(valeur(["A", "A", "9"]));
println(valeur(["10", "5", "9"]));
