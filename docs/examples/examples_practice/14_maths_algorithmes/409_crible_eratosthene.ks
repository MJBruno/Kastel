// ==================================================================
// Exemple 409 — Crible d'Ératosthène
// Catégorie : Maths et algorithmes
// ------------------------------------------------------------------
// Barrer les multiples de chaque premier.
// ------------------------------------------------------------------
// Sortie attendue :
//   [2, 3, 5, 7, 11, 13, 17, 19, 23, 29]
// ==================================================================

let n = 30;
let est_premier = [];
for i in range(n + 1) {
    est_premier.add(true);
}
est_premier[0] = false;
est_premier[1] = false;

for i in range(2, n + 1) {
    if est_premier[i] {
        let j = i * i;
        while j <= n {
            est_premier[j] = false;
            j += i;
        }
    }
}

let premiers = [];
for i in range(n + 1) {
    if est_premier[i] {
        premiers.add(i);
    }
}
println(premiers);
