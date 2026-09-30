// ==================================================================
// Exemple 457 — Mots les plus fréquents
// Catégorie : Mini-projets
// ------------------------------------------------------------------
// Compter, puis chercher le maximum.
// ------------------------------------------------------------------
// Sortie attendue :
//   le apparaît 3 fois
// ==================================================================

let texte = "le chat et le chien et le poisson";
let compte = dict();
for m in texte.split(" ") {
    compte[m] = compte.get_or(m, 0) + 1;
}

let meilleur = "";
let max_n = 0;
for m in compte {
    if compte[m] > max_n {
        max_n = compte[m];
        meilleur = m;
    }
}
println("{} apparaît {} fois", meilleur, max_n);
