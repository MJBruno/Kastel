// ==================================================================
// Exemple 580 — Annuaire téléphonique
// Catégorie : Mini-projets : objets, structures, systèmes
// ------------------------------------------------------------------
// Projet : recherche par préfixe dans un dict trié.
// ------------------------------------------------------------------
// Sortie attendue :
//   ["Ada", "Alan"]
//   ["Grace"]
//   []
// ==================================================================

let contacts = {"Ada": "0102", "Alan": "0304", "Bob": "0506", "Grace": "0708"};

func chercher(prefixe: str) -> List<str> {
    let res = contacts.keys().filter(n => n.starts_with(prefixe));
    res.sort();
    return res;
}

println(chercher("A"));
println(chercher("Gr"));
println(chercher("Z"));
