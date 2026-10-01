// ==================================================================
// Exemple 047 — Assembler avec join()
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// sep.join(liste) est l'inverse de split.
// ------------------------------------------------------------------
// Sortie attendue :
//   un, deux, trois
//   un-deux-trois
// ==================================================================

let mots = ["un", "deux", "trois"];
println(", ".join(mots));
println("-".join(mots));
