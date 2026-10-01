// ==================================================================
// Exemple 538 — Algorithme de Luhn
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : valider un numéro de carte : doubler un chiffre sur deux en partant de la droite.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
// ==================================================================

func luhn(s: str) -> bool {
    let somme = 0;
    let doubler = false;
    for i in range(s.size() - 1, -1, -1) {
        let d = int(s.char_at(i));
        if doubler {
            d *= 2;
            if d > 9 { d -= 9; }
        }
        somme += d;
        doubler = !doubler;
    }
    return somme % 10 == 0;
}

println(luhn("79927398713"));
println(luhn("79927398710"));
