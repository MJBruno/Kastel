// ==================================================================
// Exemple 194 — Inverser un dict
// Catégorie : Dict, Set, Tuple, Record
// ------------------------------------------------------------------
// Échanger clés et valeurs (les valeurs doivent être des textes ici).
// ------------------------------------------------------------------
// Sortie attendue :
//   chien
// ==================================================================

let fr_en = {"chat": "cat", "chien": "dog"};
let en_fr = dict();
for k in fr_en {
    en_fr[fr_en[k]] = k;
}
println(en_fr["dog"]);
