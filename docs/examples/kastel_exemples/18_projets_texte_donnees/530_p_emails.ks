// ==================================================================
// Exemple 530 — Extraire les adresses e-mail d'un texte
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : garder les mots contenant @ suivi d'un point.
// ------------------------------------------------------------------
// Sortie attendue :
//   ada@exemple.fr
//   bob@test.org
// ==================================================================

let texte = "contact: ada@exemple.fr, bob@test.org ou rien@ici";

for mot in texte.split(" ") {
    let m = mot;
    while m.ends_with(",") || m.ends_with(".") {
        m = m.slice(0, m.size() - 1);
    }
    let a = m.index_of("@");
    if a > 0 && m.slice(a, m.size()).contains(".") {
        println(m);
    }
}
