// ==================================================================
// Exemple 293 — finally s'exécute toujours
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// Avec ou sans erreur, finally passe en dernier.
// ------------------------------------------------------------------
// Sortie attendue :
//   succès
//   nettoyage
//   catch : raté
//   nettoyage
// ==================================================================

func tester(echec: bool) {
    try {
        if echec {
            throw "raté";
        }
        println("succès");
    } catch (e) {
        println("catch : " + e);
    } finally {
        println("nettoyage");
    }
}

tester(false);
tester(true);
