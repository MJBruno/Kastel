// ==================================================================
// Exemple 118 — Fermetures indépendantes
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// Chaque appel de la fabrique crée son propre état.
// ------------------------------------------------------------------
// Sortie attendue :
//   3
//   1
// ==================================================================

func creer_compteur() {
    let n = 0;
    return func() {
        n += 1;
        return n;
    };
}

let a = creer_compteur();
let b = creer_compteur();
a();
a();
println(a());   // 3
println(b());   // 1
