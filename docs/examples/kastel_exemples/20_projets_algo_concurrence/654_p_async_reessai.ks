// ==================================================================
// Exemple 654 — Réessayer une opération asynchrone
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : une tâche instable réussit à la 3e tentative ; l'appelant réessaie jusqu'à 5 fois.
// ------------------------------------------------------------------
// Sortie attendue :
//   ok
//   3
// ==================================================================

let essais = 0;

async func instable() -> str {
    essais += 1;
    if essais < 3 {
        throw "échec temporaire";
    }
    return "ok";
}

async func avec_reessai() -> str {
    let erreurs = 0;
    while true {
        try {
            return await instable();
        } catch (e) {
            erreurs += 1;
            if erreurs >= 5 {
                throw "abandon";
            }
        }
    }
    return "";
}

println(await avec_reessai());
println(essais);
