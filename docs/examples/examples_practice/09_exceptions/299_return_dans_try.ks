// ==================================================================
// Exemple 299 — return dans try : finally exécuté
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// La valeur retournée est conservée, finally passe avant de sortir.
// ------------------------------------------------------------------
// Sortie attendue :
//   1
//   ["finally"]
// ==================================================================

let journal = [];

func f() -> int {
    try {
        return 1;
    } finally {
        journal.add("finally");
    }
}

println(f());
println(journal);
