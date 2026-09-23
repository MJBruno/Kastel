```javascript
// examples/set_demo.ks
//
// Set : ensemble MUTABLE d'éléments UNIQUES, sans ordre garanti.
//
//   List   : mutable, ordonné, doublons permis      -> référence + copy()
//   Tuple  : immuable, ordonné, structure fixe       -> valeur immuable
//   Dict   : mutable, clé/valeur, clés uniques       -> référence + copy()
//   Set    : mutable, éléments uniques, non ordonné  -> référence + copy()
//
// Lancer avec : kastel examples/set_demo.ks

// -- API de base --------------------------------------------------------------
let s = Set(1, 2, 3);

s.add(4);
s.add(3);                               // déjà présent : sans effet

println(s.size());                      // 4
println(s.contains(2));                 // true

s.remove(2);

println(s.contains(2));                 // false
println(s.is_empty());                  // false

s.clear();

println(s.size());                      // 0
println(s.is_empty());                  // true
println(s);                             // Set()

// -- Unicité ---------------------------------------------------------------------
let u = Set(1, 2, 2, 3, 3, 3);

println(u.size());                      // 3
println(u);                             // {1, 2, 3}

// Un tuple est comparé par son CONTENU.
let points = Set((1, 2), (1, 2), (2, 1));

println(points.size());                 // 2

// -- Littéral {...} -----------------------------------------------------------------
// `{1, 2, 3}` est un ensemble ; `{"clé": valeur}` un dict ; `{clé: valeur}`
// un record ; `{}` est le dict vide (l'ensemble vide s'écrit Set()).
let lettres: Set<str> = {"a", "b", "c"};
let ages = { "age": 25 };

println(lettres.size());                // 3
println(ages);                          // {"age": 25}

// -- Opérations ensemblistes (elles renvoient un NOUVEL ensemble) ------------
let a = Set(1, 2, 3);
let b = Set(3, 4, 5);

println(a.union(b));                    // {1, 2, 3, 4, 5}
println(a.intersection(b));             // {3}
println(a.difference(b));               // {1, 2}
println(a.symmetric_difference(b));     // {1, 2, 4, 5}

println(Set(1, 2).is_subset(a));        // true
println(a.is_superset(Set(1, 2)));      // true
println(a.is_subset(b));                // false

// -- Référence / copie -----------------------------------------------------------------
let x = {1, 2, 3};
let y = x;                              // MÊME objet

y.add(4);

println(x.contains(4));                 // true

let z = x.copy();                       // copie explicite

z.add(5);

println(x.contains(5));                 // false
println(z.contains(5));                 // true

// -- Parcours -----------------------------------------------------------------------------
// L'ordre n'est pas garanti : ne vous y fiez pas.
let total = 0;

for item in Set(10, 20, 30) {
    total = total + item;
}

println(total);                         // 60
println(type(a));                       // set

```