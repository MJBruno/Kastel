// ==================================================================
// Exemple 252 — Alias de type
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// type Nom = ... donne un nom à un type composé.
// ------------------------------------------------------------------
// Sortie attendue :
//   Ada (36)
// ==================================================================

type Personne = { nom: str, age: int };

func presenter(p: Personne) -> str {
    return p.nom + " (" + str(p.age) + ")";
}

println(presenter({ nom: "Ada", age: 36 }));
