// ==================================================================
// Exemple 102 — Définir et appeler une fonction
// Catégorie : Fonctions, fermetures, récursion
// ------------------------------------------------------------------
// func nom(parametres) { ... } puis nom(arguments).
// ------------------------------------------------------------------
// Sortie attendue :
//   Bonjour, Ada !
//   Bonjour, Alan !
// ==================================================================

func saluer(nom: str) {
    println("Bonjour, " + nom + " !");
}

saluer("Ada");
saluer("Alan");
