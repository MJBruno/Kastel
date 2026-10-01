// ==================================================================
// Exemple 653 — Téléchargements asynchrones
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : trois « téléchargements » lancés ensemble, octets totaux récupérés avec await.
// ------------------------------------------------------------------
// Sortie attendue :
//   350 octets
// ==================================================================

async func telecharger(nom: str, octets: int, delai: int) -> int {
    sleep(delai);
    return octets;
}

let a = telecharger("a.zip", 100, 30);
let b = telecharger("b.zip", 200, 10);
let c = telecharger("c.zip", 50, 20);

let total = (await a) + (await b) + (await c);
println("{} octets", total);
