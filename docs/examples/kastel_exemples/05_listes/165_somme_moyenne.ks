// ==================================================================
// Exemple 165 — Somme et moyenne
// Catégorie : Listes
// ------------------------------------------------------------------
// Un accumulateur, puis une division (résultat float).
// ------------------------------------------------------------------
// Sortie attendue :
//   54
//   13.5
// ==================================================================

let notes = [12, 15, 9, 18];
let somme = 0;
for n in notes {
    somme += n;
}
println(somme);
println(somme / notes.size());
