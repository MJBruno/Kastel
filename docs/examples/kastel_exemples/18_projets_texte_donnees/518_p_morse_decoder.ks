// ==================================================================
// Exemple 518 — Code Morse : décoder
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : inverser la table pour retrouver les lettres.
// ------------------------------------------------------------------
// Sortie attendue :
//   SOS
//   AT
// ==================================================================

let table = {"S": "...", "O": "---", "A": ".-", "T": "-", "E": "."};
let inverse = dict();
for lettre in table {
    inverse[table[lettre]] = lettre;
}

func decoder(morse: str) -> str {
    let res = "";
    for code in morse.split(" ") {
        res += inverse[code];
    }
    return res;
}

println(decoder("... --- ..."));
println(decoder(".- -"));
