// ==================================================================
// Exemple 285 — Traiter une liste de Result
// Catégorie : match, Option, Result
// ------------------------------------------------------------------
// Compter succès et échecs.
// ------------------------------------------------------------------
// Sortie attendue :
//   3 ok, 2 erreurs
// ==================================================================

func valider(n: int) -> Result<int, str> {
    if n < 0 {
        return Err("négatif");
    }
    return Ok(n);
}

let bons = 0;
let mauvais = 0;
for n in [3, -1, 8, -5, 0] {
    if valider(n).is_ok() {
        bons += 1;
    } else {
        mauvais += 1;
    }
}
println("{} ok, {} erreurs", bons, mauvais);
