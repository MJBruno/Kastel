// ==================================================================
// Exemple 556 — Retour à la ligne automatique
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : couper un texte en lignes de 15 caractères maximum, sans couper de mot.
// ------------------------------------------------------------------
// Sortie attendue :
//   le langage
//   kastel est
//   simple et
//   rapide
// ==================================================================

func couper(texte: str, largeur: int) -> List<str> {
    let lignes = [];
    let courante = "";
    for mot in texte.split(" ") {
        if courante == "" {
            courante = mot;
        } else if courante.size() + 1 + mot.size() <= largeur {
            courante += " " + mot;
        } else {
            lignes.add(courante);
            courante = mot;
        }
    }
    lignes.add(courante);
    return lignes;
}

for l in couper("le langage kastel est simple et rapide", 15) {
    println(l);
}
