// ==================================================================
// Exemple 239 — Les objets sont partagés
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// Deux variables peuvent désigner le même objet.
// ------------------------------------------------------------------
// Sortie attendue :
//   plein
// ==================================================================

class Boite {
    let contenu: str = "vide";
}

let a = new Boite();
let b = a;
b.contenu = "plein";
println(a.contenu);
