// ==================================================================
// Exemple 507 — Course de voitures
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : accélérations et vitesse maximale, qui gagne sur 100 mètres ?
// ------------------------------------------------------------------
// Sortie attendue :
//   12
//   13
//   14
// ==================================================================

func temps(acceleration: int, vmax: int) -> int {
    let v = 0;
    let d = 0;
    let t = 0;
    while d < 100 {
        v = min(v + acceleration, vmax);
        d += v;
        t += 1;
    }
    return t;
}

println(temps(2, 10));
println(temps(5, 8));
println(temps(1, 30));
