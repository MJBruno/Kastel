// ====================================================================
// Kastel — Alias de type
// Notions : type Nom = { ... }, paramètre typé par l'alias
// Résultat attendu :
//   Bruno (25 ans)
// ====================================================================

type Personne = { nom: str, age: int };

func presenter(p: Personne) -> str {
    return p.nom + " (" + str(p.age) + " ans)";
}

let bruno: Personne = { nom: "Bruno", age: 25 };
println(presenter(bruno));
