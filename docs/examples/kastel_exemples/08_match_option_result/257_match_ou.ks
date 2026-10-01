// ==================================================================
// Exemple 257 — Motifs alternatifs avec |
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Plusieurs valeurs dans un même bras.
// ------------------------------------------------------------------
// Sortie attendue :
//   week-end
//   semaine
// ==================================================================

func type_de_jour(n: int) -> str {
    match n {
        6 | 7 => { return "week-end"; }
        _ => { return "semaine"; }
    }
}

println(type_de_jour(6));
println(type_de_jour(3));
