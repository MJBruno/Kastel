// ==================================================================
// Exemple 274 — L'opérateur ? sur Option
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// expr? renvoie la valeur, ou termine la fonction avec None.
// ------------------------------------------------------------------
// Sortie attendue :
//   Some(8)
//   None
// ==================================================================

func premier_pair(v: List<int>) -> Option<int> {
    for x in v {
        if x % 2 == 0 {
            return Some(x);
        }
    }
    return None;
}

func double_du_premier_pair(v: List<int>) -> Option<int> {
    let p = premier_pair(v)?;
    return Some(p * 2);
}

println(double_du_premier_pair([1, 3, 4, 6]));
println(double_du_premier_pair([1, 3, 5]));
