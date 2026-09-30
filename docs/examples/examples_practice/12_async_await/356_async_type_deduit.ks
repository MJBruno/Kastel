// ==================================================================
// Exemple 356 — Type de retour déduit
// Catégorie : async / await
// ------------------------------------------------------------------
// Sans -> type, le Task<T> est déduit du return.
// ------------------------------------------------------------------
// Sortie attendue :
//   prêt
// ==================================================================

async func texte() {
    return "prêt";
}

let t: Task<str> = texte();
println(await t);
