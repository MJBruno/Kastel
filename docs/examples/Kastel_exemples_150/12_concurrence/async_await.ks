// ====================================================================
// Kastel — async et await
// Notions : async func produit une Task, await en lit le résultat
// Résultat attendu :
//   42
//   Bonjour Kastel
// ====================================================================

async func calculer() -> int {
    yield();
    return 42;
}

async func saluer(nom: str) -> str {
    return "Bonjour " + nom;
}

let t1 = calculer();
let t2 = saluer("Kastel");

println(await t1);
println(await t2);
