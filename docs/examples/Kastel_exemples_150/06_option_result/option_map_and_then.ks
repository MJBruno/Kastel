// ====================================================================
// Kastel — Transformer une Option
// Notions : map, and_then
// Résultat attendu :
//   Some(4)
//   None
//   Some(4)
// ====================================================================

func moitie(n: int) -> Option<int> {
    if n % 2 == 0 {
        return Some(idiv(n, 2));
    }
    return None;
}

println(Some(8).and_then(moitie));
println(Some(7).and_then(moitie));
println(Some(3).map(x => x + 1));
