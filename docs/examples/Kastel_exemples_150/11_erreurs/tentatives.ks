// ====================================================================
// Kastel — Réessayer une opération
// Notions : boucle + try/catch, compteur de tentatives
// Résultat attendu :
//   tentative 1 : échec
//   tentative 2 : échec
//   tentative 3 : succès
// ====================================================================

func operation(essai: int) {
    if essai < 3 {
        throw "indisponible";
    }
    return "succès";
}

let essai = 1;
let termine = false;

while !termine {
    try {
        let r = operation(essai);
        println("tentative {} : {}", essai, r);
        termine = true;
    } catch (e) {
        println("tentative {} : échec", essai);
        essai += 1;
    }
}
