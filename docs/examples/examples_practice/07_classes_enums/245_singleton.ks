// ==================================================================
// Exemple 245 — Un singleton par static
// Catégorie : Classes, interfaces, enums, alias
// ------------------------------------------------------------------
// Une seule instance partagée, créée à la demande.
// ------------------------------------------------------------------
// Sortie attendue :
//   7
// ==================================================================

class Config {
    static let instance = None;
    let valeur: int = 42;

    static func obtenir() -> Config {
        if Config.instance == None {
            Config.instance = new Config();
        }
        return Config.instance;
    }
}

let a = Config.obtenir();
let b = Config.obtenir();
a.valeur = 7;
println(b.valeur);
