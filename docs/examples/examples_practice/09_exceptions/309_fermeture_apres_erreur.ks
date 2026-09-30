// ==================================================================
// Exemple 309 — Une fermeture survit à une exception
// Catégorie : Exceptions try/catch/finally
// ------------------------------------------------------------------
// La valeur capturée dans le try reste valide après le catch.
// ------------------------------------------------------------------
// Sortie attendue :
//   42
// ==================================================================

let sauvegarde = [];

func executer() {
    try {
        let x = 41;
        func lire() -> int {
            return x + 1;
        }
        sauvegarde.add(lire);
        throw "erreur";
    } catch (e) {
        let a = 1000;
        let b = 2000;
    }
}

executer();
let f = sauvegarde[0];
println(f());
