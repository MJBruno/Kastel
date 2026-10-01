// ==================================================================
// Exemple 061 — Compter les occurrences d'un caractère
// Catégorie : Chaînes de caractères
// ------------------------------------------------------------------
// Un compteur simple avec for.
// ------------------------------------------------------------------
// Sortie attendue :
//   4
// ==================================================================

let s = "mississippi";
let n = 0;
for c in s {
    if c == "s" {
        n += 1;
    }
}
println(n);
