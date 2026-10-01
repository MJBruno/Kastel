// ==================================================================
// Exemple 090 — Jeu de devinette (simulé)
// Catégorie : Structures de contrôle
// ------------------------------------------------------------------
// Une boucle while qui cherche un nombre secret par dichotomie.
// ------------------------------------------------------------------
// Sortie attendue :
//   trouvé en 3 essais
// ==================================================================

let secret = 37;
let bas = 1;
let haut = 100;
let essais = 0;
while true {
    let milieu = idiv(bas + haut, 2);
    essais += 1;
    if milieu == secret {
        break;
    } else if milieu < secret {
        bas = milieu + 1;
    } else {
        haut = milieu - 1;
    }
}
println("trouvé en {} essais", essais);
