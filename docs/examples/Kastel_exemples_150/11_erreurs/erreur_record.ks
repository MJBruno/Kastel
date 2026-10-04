// ====================================================================
// Kastel — Erreurs structurées
// Notions : lever un record { code, message }
// Résultat attendu :
//   Erreur 404 : introuvable
//   Erreur 500 : serveur
// ====================================================================

func requete(code: int) {
    if code == 404 {
        throw { code: 404, message: "introuvable" };
    }
    if code == 500 {
        throw { code: 500, message: "serveur" };
    }
    return "ok";
}

for c in [404, 500] {
    try {
        println(requete(c));
    } catch (e) {
        println("Erreur {} : {}", e.code, e.message);
    }
}
