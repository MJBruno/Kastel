// ==================================================================
// Exemple 535 — Encodage Base64
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : regrouper les octets par trois et les découper en blocs de 6 bits.
// ------------------------------------------------------------------
// Sortie attendue :
//   TWFu
//   TWE=
//   TQ==
// ==================================================================

let minuscules = "abcdefghijklmnopqrstuvwxyz";
let majuscules = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
let chiffres = "0123456789";

// Code ASCII d'un caractère usuel (lettres, chiffres, espace).
func code(c: str) -> int {
    if c == " " { return 32; }
    let i = majuscules.index_of(c);
    if i != -1 { return 65 + i; }
    i = minuscules.index_of(c);
    if i != -1 { return 97 + i; }
    return 48 + chiffres.index_of(c);
}

       let b64 = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

       func base64(texte: str) -> str {
           let res = "";
           let i = 0;
           while i < texte.size() {
               let restants = min(3, texte.size() - i);
               let b1 = code(texte.char_at(i));
               let b2 = restants > 1 ? code(texte.char_at(i + 1)) : 0;
               let b3 = restants > 2 ? code(texte.char_at(i + 2)) : 0;
               let n = (b1 << 16) | (b2 << 8) | b3;
               res += b64.char_at((n >> 18) & 63);
               res += b64.char_at((n >> 12) & 63);
               res += restants > 1 ? b64.char_at((n >> 6) & 63) : "=";
               res += restants > 2 ? b64.char_at(n & 63) : "=";
               i += 3;
           }
           return res;
       }

       println(base64("Man"));
       println(base64("Ma"));
       println(base64("M"));
