// ==================================================================
// Exemple 101 — Somme des chiffres d'un nombre
// Catégorie : Structures de contrôle
// ------------------------------------------------------------------
// n % 10 donne le dernier chiffre, idiv(n, 10) retire ce chiffre.
// ------------------------------------------------------------------
// Sortie attendue :
//   15
// ==================================================================

let n = 12345;
let somme = 0;
while n > 0 {
    somme += n % 10;
    n = idiv(n, 10);
}
println(somme);
