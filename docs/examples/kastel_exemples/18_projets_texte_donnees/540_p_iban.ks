// ==================================================================
// Exemple 540 — Validation d'un IBAN
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : déplacer les 4 premiers caractères, convertir les lettres en nombres, puis modulo 97.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
// ==================================================================

let majuscules = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";

func iban_valide(s: str) -> bool {
    let r = s.slice(4, s.size()) + s.slice(0, 4);
    let chiffres = "";
    for c in r {
        let i = majuscules.index_of(c);
        chiffres += i == -1 ? c : str(i + 10);   // A = 10, B = 11...
    }
    let reste = 0;
    for c in chiffres {
        reste = (reste * 10 + int(c)) % 97;
    }
    return reste == 1;
}

println(iban_valide("GB82WEST12345698765432"));
println(iban_valide("GB82WEST12345698765433"));
