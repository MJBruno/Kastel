// ==================================================================
// Exemple 522 — Nombre en lettres (français)
// Catégorie : Mini-projets : texte, données, calculs
// ------------------------------------------------------------------
// Projet : écrire les nombres de 0 à 99, avec les règles de soixante-dix et quatre-vingts.
// ------------------------------------------------------------------
// Sortie attendue :
//   0 : zéro
//   17 : dix-sept
//   21 : vingt-et-un
//   42 : quarante-deux
//   71 : soixante-et-onze
//   80 : quatre-vingts
//   99 : quatre-vingt-dix-neuf
// ==================================================================

let unites = ["zéro", "un", "deux", "trois", "quatre", "cinq", "six", "sept", "huit", "neuf",
              "dix", "onze", "douze", "treize", "quatorze", "quinze", "seize",
              "dix-sept", "dix-huit", "dix-neuf"];
let dizaines = ["", "", "vingt", "trente", "quarante", "cinquante", "soixante"];

func lettres(n: int) -> str {
    if n < 20 { return unites[n]; }
    if n < 70 {
        let base = dizaines[idiv(n, 10)];
        let r = n % 10;
        if r == 0 { return base; }
        if r == 1 { return base + "-et-un"; }
        return base + "-" + unites[r];
    }
    if n < 80 {
        if n == 71 { return "soixante-et-onze"; }
        return "soixante-" + unites[n - 60];
    }
    if n == 80 { return "quatre-vingts"; }
    return "quatre-vingt-" + unites[n - 80];
}

for n in [0, 17, 21, 42, 71, 80, 99] {
    println("{} : {}", n, lettres(n));
}
