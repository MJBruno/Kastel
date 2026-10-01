// ==================================================================
// Exemple 036 — clock() : mesurer le temps
// Catégorie : Bases du langage
// ------------------------------------------------------------------
// clock() renvoie un flottant ; deux appels permettent de mesurer une durée.
// ------------------------------------------------------------------
// Sortie attendue :
//   499500
//   true
// ==================================================================

let debut = clock();
let total = 0;
for i in range(1000) {
    total += i;
}
let duree = clock() - debut;
println(total);
println(duree >= 0);
