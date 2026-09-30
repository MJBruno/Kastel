// ==================================================================
// Exemple 247 — Une énumération
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// enum liste des valeurs possibles ; on y accède par Enum.Valeur.
// ------------------------------------------------------------------
// Sortie attendue :
//   true
//   false
// ==================================================================

enum Couleur {
    Rouge,
    Vert,
    Bleu
}

let c = Couleur.Vert;
println(c == Couleur.Vert);
println(c == Couleur.Bleu);
