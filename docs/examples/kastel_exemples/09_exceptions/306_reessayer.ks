// ==================================================================
// Exemple 306 — Réessayer après une erreur
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Une boucle qui retente jusqu'au succès.
// ------------------------------------------------------------------
// Sortie attendue :
//   nouvelle tentative
//   nouvelle tentative
//   réussi
//   3
// ==================================================================

let tentatives = 0;

func operation_instable() -> str {
    tentatives += 1;
    if tentatives < 3 {
        throw "échec temporaire";
    }
    return "réussi";
}

let resultat = "";
while resultat == "" {
    try {
        resultat = operation_instable();
    } catch (e) {
        println("nouvelle tentative");
    }
}
println(resultat);
println(tentatives);
