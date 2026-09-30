// ==================================================================
// Exemple 193 — Dict imbriqué
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// Une valeur de dict peut être un autre dict ou une liste.
// ------------------------------------------------------------------
// Sortie attendue :
//   kastel
//   Paris
// ==================================================================

let user = {
    "nom": "Ada",
    "langages": ["kastel", "rust"],
    "adresse": {"ville": "Paris"}
};
println(user["langages"][0]);
println(user["adresse"]["ville"]);
