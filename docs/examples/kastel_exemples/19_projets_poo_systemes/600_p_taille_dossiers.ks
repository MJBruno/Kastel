// ==================================================================
// Exemple 600 — Taille des dossiers
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : additionner les fichiers dont le chemin commence par un dossier.
// ------------------------------------------------------------------
// Sortie attendue :
//   total : 22
//   d : 12
//   d/c : 7
// ==================================================================

let fichiers = [("a.txt", 10), ("d/b.txt", 5), ("d/c/e.txt", 7)];

func taille(dossier: str) -> int {
    let total = 0;
    for f in fichiers {
        if dossier == "" || f[0].starts_with(dossier + "/") {
            total += f[1];
        }
    }
    return total;
}

println("total : {}", taille(""));
println("d : {}", taille("d"));
println("d/c : {}", taille("d/c"));
