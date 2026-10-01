// ==================================================================
// Exemple 501 — Simon : mémoriser une séquence
// Catégorie : Mini-projets : jeux et simulations
// ------------------------------------------------------------------
// Projet : comparer la séquence du joueur à celle de l'ordinateur, étape par étape.
// ------------------------------------------------------------------
// Sortie attendue :
//   erreur à l'étape 3
//   2 bonnes réponses
// ==================================================================

let sequence = ["rouge", "vert", "bleu", "vert"];
let joueur = ["rouge", "vert", "jaune"];

let ok = 0;
for i in range(joueur.size()) {
    if joueur[i] == sequence[i] {
        ok += 1;
    } else {
        println("erreur à l'étape {}", i + 1);
        break;
    }
}
println("{} bonnes réponses", ok);
