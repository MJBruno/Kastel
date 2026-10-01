// ==================================================================
// Exemple 462 — Plus court chemin dans une grille
// Catégorie : Mini-projets
// ------------------------------------------------------------------
// Largeur d'abord sur une petite grille (# = mur).
// ------------------------------------------------------------------
// Sortie attendue :
//   4
// ==================================================================

let grille = [
    "..#",
    ".#.",
    "..."
];

let dist = dict();
dist["0,0"] = 0;
let file = [(0, 0)];

while !file.is_empty() {
    let cur = file.remove_at(0);
    let x = cur[0];
    let y = cur[1];
    let d = dist[str(x) + "," + str(y)];
    for dir in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        let nx = x + dir[0];
        let ny = y + dir[1];
        if nx >= 0 && nx < 3 && ny >= 0 && ny < 3 {
            let cle = str(nx) + "," + str(ny);
            if grille[ny].char_at(nx) == "." && !dist.contains(cle) {
                dist[cle] = d + 1;
                file.add((nx, ny));
            }
        }
    }
}
println(dist["2,2"]);
