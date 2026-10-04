// ====================================================================
// Kastel — Types union
// Notions : int | str dans une signature
// Résultat attendu :
//   valeur : 7
//   valeur : sept
// ====================================================================

func decrire(x: int | str) -> str {
    return "valeur : " + str(x);
}

println(decrire(7));
println(decrire("sept"));
