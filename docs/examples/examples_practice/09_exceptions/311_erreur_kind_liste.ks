// ==================================================================
// Exemple 311 — Distinguer plusieurs erreurs
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Tester e.kind pour réagir différemment.
// ------------------------------------------------------------------
// Sortie attendue :
//   division
//   index
//   autre : TypeError
// ==================================================================

func tenter(n: int) {
    try {
        if n == 0 { let x = 1 / 0; }
        if n == 1 { let y = [1][5]; }
        if n == 2 { let z = "abc".to_int(); }
    } catch (e: Err) {
        if e.kind == "DivisionByZero" {
            println("division");
        } else if e.kind == "ArrayIndexOutOfBounds" {
            println("index");
        } else {
            println("autre : " + e.kind);
        }
    }
}

tenter(0);
tenter(1);
tenter(2);
