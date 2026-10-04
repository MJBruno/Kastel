// ====================================================================
// Kastel — Énumérations
// Notions : enum, Enum.Variant, comparaison avec ==
// Résultat attendu :
//   Couleur.Vert
//   true
//   false
// ====================================================================

enum Couleur {
    Rouge,
    Vert,
    Bleu
}

let c: Couleur = Couleur.Vert;

println(c);
println(c == Couleur.Vert);
println(c == Couleur.Bleu);
