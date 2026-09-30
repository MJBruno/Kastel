// ==================================================================
// Exemple 335 — Interface générique
// Catégorie : Génériques et surcharge d'opérateurs
// ------------------------------------------------------------------
// Comparable<T> : la méthode dépend du type T choisi.
// ------------------------------------------------------------------
// Sortie attendue :
//   6
// ==================================================================

interface Comparable<T> {
    func comparer(autre: T) -> int;
}

class Nombre: Comparable<int> {
    let v: int = 10;
    func comparer(autre: int) -> int {
        return self.v - autre;
    }
}

println(new Nombre().comparer(4));
