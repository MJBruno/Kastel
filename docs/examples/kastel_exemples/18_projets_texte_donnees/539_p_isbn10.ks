// ==================================================================
// Exemple 539 — Validation d'un ISBN-10
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : somme pondérée des chiffres, divisible par 11.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
// ==================================================================

func isbn10(s: str) -> bool {
    let somme = 0;
    for i in range(10) {
        let c = s.char_at(i);
        let v = c == "X" ? 10 : int(c);
        somme += v * (10 - i);
    }
    return somme % 11 == 0;
}

println(isbn10("0306406152"));
println(isbn10("0306406153"));
