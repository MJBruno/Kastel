// ==================================================================
// Exemple 117 — Fermeture : un compteur
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// La fonction interne se souvient de la variable de la fonction externe.
// ------------------------------------------------------------------
// Sortie attendue :
//   1
//   2
//   3
// ==================================================================

func creer_compteur() {
    let n = 0;
    return func() {
        n += 1;
        return n;
    };
}

let c = creer_compteur();
println(c());
println(c());
println(c());
