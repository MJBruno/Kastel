// ==================================================================
// Exemple 517 — Code Morse : encoder
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : traduire un mot en points et traits (table réduite).
// ------------------------------------------------------------------
// Sortie attendue :
//   ... --- ...
//   .- -
// ==================================================================

let table = {"S": "...", "O": "---", "A": ".-", "T": "-", "E": "."};

func vers_morse(mot: str) -> str {
    let codes = [];
    for c in mot {
        codes.add(table[c]);
    }
    return " ".join(codes);
}

println(vers_morse("SOS"));
println(vers_morse("AT"));
