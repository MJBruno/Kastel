// ==================================================================
// Exemple 350 — await directement sur l'appel
// Catégorie : async / await
// ------------------------------------------------------------------
// On peut écrire await f(...) sans variable intermédiaire.
// ------------------------------------------------------------------
// Sortie attendue :
//   Bonjour Ada
// ==================================================================

async func bonjour(nom: str) -> str {
    return "Bonjour " + nom;
}

println(await bonjour("Ada"));
