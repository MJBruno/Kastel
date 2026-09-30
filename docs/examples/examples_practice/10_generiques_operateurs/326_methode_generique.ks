// ==================================================================
// Exemple 326 — Méthode générique dans une classe
// Catégorie : Génériques et surcharge d'opérateurs
// ------------------------------------------------------------------
// Une méthode peut avoir son propre paramètre de type.
// ------------------------------------------------------------------
// Sortie attendue :
//   10
//   kastel
// ==================================================================

class Outils {
    func echo<T>(x: T) -> T {
        return x;
    }
}

let o = new Outils();
let a: int = o.echo(10);
let b: str = o.echo<str>("kastel");
println(a);
println(b);
