// ==================================================================
// Exemple 536 — Affichage hexadécimal
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : les codes des caractères en hexadécimal sur deux chiffres.
// ------------------------------------------------------------------
// Sortie attendue :
//   4b 61 73 74 65 6c
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

       let parties = [];
       for c in "Kastel" {
           parties.add(format("{:02x}", code(c)));
       }
       println(" ".join(parties));
