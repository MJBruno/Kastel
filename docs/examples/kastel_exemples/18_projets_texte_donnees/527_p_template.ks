// ==================================================================
// Exemple 527 — Moteur de modèles
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : remplacer {{nom}} par des valeurs.
// ------------------------------------------------------------------
// Sortie attendue :
//   Bonjour Ada, vous avez 3 messages
// ==================================================================

func rendre(modele: str, valeurs) -> str {
    let res = modele;
    for cle in valeurs {
        res = res.replace_all("{{" + cle + "}}", str(valeurs[cle]));
    }
    return res;
}

println(rendre("Bonjour {{nom}}, vous avez {{n}} messages", {"nom": "Ada", "n": 3}));
