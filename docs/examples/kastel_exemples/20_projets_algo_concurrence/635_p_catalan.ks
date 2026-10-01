// ==================================================================
// Exemple 635 — Nombres de Catalan
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : C(n) = somme des C(i) * C(n-1-i).
// ------------------------------------------------------------------
// Sortie attendue :
//   [1, 1, 2, 5, 14, 42]
// ==================================================================

let c = [1];
for n in range(1, 6) {
    let s = 0;
    for i in range(n) {
        s += c[i] * c[n - 1 - i];
    }
    c.add(s);
}
println(c);
