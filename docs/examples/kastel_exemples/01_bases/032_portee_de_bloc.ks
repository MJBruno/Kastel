// ==================================================================
// Exemple 032 — Portée d'un bloc { }
// Catégorie : Bases du langage
// ------------------------------------------------------------------
// Une variable déclarée dans un bloc n'existe que dans ce bloc.
// ------------------------------------------------------------------
// Sortie attendue :
//   2
//   1
// ==================================================================

let x = 1;
{
    let x = 2;       // nouvelle variable, locale au bloc
    println(x);
}
println(x);          // la variable externe est intacte
