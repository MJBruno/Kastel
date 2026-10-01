// ==================================================================
// Exemple 488 — Serpents et échelles
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : avancer un pion sur 30 cases avec des raccourcis (échelles) et des chutes (serpents).
// ------------------------------------------------------------------
// Sortie attendue :
//   29
// ==================================================================

let sauts = {"3": 11, "6": 17, "9": 18, "14": 4, "19": 8, "22": 5, "27": 1};
let des = [3, 4, 2, 6, 5, 1, 4, 3, 6, 6, 6];

let pos = 0;
for d in des {
    pos += d;
    if pos > 30 { pos -= d; continue; }
    if sauts.contains(str(pos)) {
        pos = sauts[str(pos)];
    }
    if pos == 30 { break; }
}
println(pos);
