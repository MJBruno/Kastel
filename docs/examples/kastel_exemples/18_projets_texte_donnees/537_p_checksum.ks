// ==================================================================
// Exemple 537 — Somme de contrôle
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : additionner les codes des caractères modulo 256.
// ------------------------------------------------------------------
// Sortie attendue :
//   38
//   39
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

       func somme_controle(texte: str) -> int {
           let s = 0;
           for c in texte {
               s += code(c);
           }
           return s % 256;
       }

       println(somme_controle("abc"));
       println(somme_controle("abd"));
