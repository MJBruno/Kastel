// ==================================================================
// Exemple 111 — Tours de Hanoï
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// Compter les déplacements pour n disques.
// ------------------------------------------------------------------
// Sortie attendue :
//   7
//   1023
// ==================================================================

func hanoi(n: int) -> int {
    if n == 0 {
        return 0;
    }
    return 2 * hanoi(n - 1) + 1;
}

println(hanoi(3));
println(hanoi(10));
