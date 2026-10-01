// ==================================================================
// Exemple 064 — Chiffre de César
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// Décaler chaque lettre minuscule de 3 positions dans l'alphabet.
// ------------------------------------------------------------------
// Sortie attendue :
//   erqmrxu
//   bonjour
// ==================================================================

func cesar(texte: str, decalage: int) -> str {
    let alphabet = "abcdefghijklmnopqrstuvwxyz";
    let resultat = "";
    for c in texte {
        let i = alphabet.index_of(c);
        if i == -1 {
            resultat += c;
        } else {
            resultat += alphabet.char_at((i + decalage) % 26);
        }
    }
    return resultat;
}

println(cesar("bonjour", 3));
println(cesar("erqmrxu", 23));
