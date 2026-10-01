// ==================================================================
// Exemple 604 — Validation de formulaire
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : accumuler toutes les erreurs au lieu de s'arrêter à la première.
// ------------------------------------------------------------------
// Sortie attendue :
//   nom requis
//   mineur
//   email invalide
//   formulaire valide
// ==================================================================

func valider(nom: str, age: int, email: str) -> List<str> {
    let erreurs = [];
    if nom.trim().is_empty() { erreurs.add("nom requis"); }
    if age < 18 { erreurs.add("mineur"); }
    if !email.contains("@") { erreurs.add("email invalide"); }
    return erreurs;
}

let e1 = valider("", 17, "x");
for e in e1 { println(e); }

let e2 = valider("Ada", 36, "ada@exemple.fr");
println(e2.is_empty() ? "formulaire valide" : "erreurs");
