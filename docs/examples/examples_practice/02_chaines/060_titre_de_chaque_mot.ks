// ==================================================================
// Exemple 060 — Mettre chaque mot en majuscule initiale
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// split + map sur la liste + join.
// ------------------------------------------------------------------
// Sortie attendue :
//   Le Langage Kastel
// ==================================================================

func cap(m: str) -> str {
    return m.char_at(0).upper() + m.slice(1, m.size());
}

let phrase = "le langage kastel";
let mots = phrase.split(" ").map(cap);
println(" ".join(mots));
