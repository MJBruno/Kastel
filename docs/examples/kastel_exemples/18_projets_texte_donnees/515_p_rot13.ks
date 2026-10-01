// ==================================================================
// Exemple 515 — Chiffrement ROT13
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : décaler chaque lettre de 13 rangs ; appliquer deux fois redonne le texte.
// ------------------------------------------------------------------
// Sortie attendue :
//   Uryyb, Jbeyq!
//   Hello, World!
// ==================================================================

let minuscules = "abcdefghijklmnopqrstuvwxyz";
let majuscules = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

func rot13(texte: str) -> str {
    let res = "";
    for c in texte {
        let i = minuscules.index_of(c);
        let j = majuscules.index_of(c);
        if i != -1 {
            res += minuscules.char_at((i + 13) % 26);
        } else if j != -1 {
            res += majuscules.char_at((j + 13) % 26);
        } else {
            res += c;      // ponctuation et espaces inchangés
        }
    }
    return res;
}

println(rot13("Hello, World!"));
println(rot13(rot13("Hello, World!")));
