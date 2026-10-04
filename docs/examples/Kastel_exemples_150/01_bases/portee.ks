// ====================================================================
// Kastel — Portée des variables
// Notions : variables globales, locales et blocs
// Résultat attendu :
//   lire() = 15
//   bloc : 99
//   global = 10
// ====================================================================

let global_value = 10;

func lire() {
    let local = 5;
    return global_value + local;
}

println("lire() = {}", lire());

{
    let temporaire = 99;
    println("bloc : {}", temporaire);
}

println("global = {}", global_value);
