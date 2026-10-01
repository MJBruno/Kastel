// ==================================================================
// Exemple 516 — Chiffre de Vigenère
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : chaque lettre est décalée selon la lettre correspondante d'une clé répétée.
// ------------------------------------------------------------------
// Sortie attendue :
//   LXFOPVEFRNHR
//   ATTACKATDAWN
// ==================================================================

let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

func vigenere(texte: str, cle: str, sens: int) -> str {
    let res = "";
    let k = 0;
    for c in texte {
        let i = alphabet.index_of(c);
        let d = alphabet.index_of(cle.char_at(k % cle.size())) * sens;
        res += alphabet.char_at((i + d + 26) % 26);
        k += 1;
    }
    return res;
}

let chiffre = vigenere("ATTACKATDAWN", "LEMON", 1);
println(chiffre);
println(vigenere(chiffre, "LEMON", -1));
