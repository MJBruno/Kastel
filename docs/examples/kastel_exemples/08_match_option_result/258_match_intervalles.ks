// ==================================================================
// Exemple 258 — Intervalles dans un motif
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// a..b exclut b ; a..=b l'inclut.
// ------------------------------------------------------------------
// Sortie attendue :
//   insuffisant
//   passable
//   bien
//   note invalide
// ==================================================================

func mention(note: int) -> str {
    match note {
        0..10 => { return "insuffisant"; }
        10..=14 => { return "passable"; }
        15..=20 => { return "bien"; }
        _ => { return "note invalide"; }
    }
}

println(mention(8));
println(mention(14));
println(mention(18));
println(mention(25));
