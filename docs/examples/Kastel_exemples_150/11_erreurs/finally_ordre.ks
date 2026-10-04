// ====================================================================
// Kastel — try / catch / finally
// Notions : finally s'exécute toujours, avec ou sans erreur
// Résultat attendu :
//   ouvrir
//   travail
//   fermer
//   ouvrir
//   erreur : panne
//   fermer
// ====================================================================

func tache(echoue: bool) {
    println("ouvrir");
    try {
        if echoue {
            throw "panne";
        }
        println("travail");
    } catch (e) {
        println("erreur : {}", e);
    } finally {
        println("fermer");
    }
}

tache(false);
tache(true);
