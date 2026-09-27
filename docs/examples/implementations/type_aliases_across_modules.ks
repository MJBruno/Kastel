// examples/type_aliases_across_modules.ks
//
// Un alias de type peut être exporté et importé entre modules, comme une
// classe ou une fonction :
//
//   export type Person = { name: str, age: int };
//   export type Number = int | float;
//
// L'alias n'a AUCUNE existence à l'exécution (aucune valeur "Person" n'est
// définie) : seule sa forme (le type) traverse la frontière du module.
// Voir shapes.ks à côté de ce fichier.
// Lancer avec : kastel examples/type_aliases_across_modules.ks

from shapes import Point, Number, ScaledPoint, origin, scale;

let p: Point = origin();

println(p.x);                        // 0
println(p.y);                        // 0

let moved: Point = { x: p.x + 1, y: p.y + 1 };

println(moved);                      // {x: 1, y: 1}

let n: Number = 3;
let m: Number = 2.5;

println(n + m);                      // 5.5

// Union en paramètre de fonction, importée : un `int` ou un `float`
// conviennent tous les deux pour `factor`.
let scaled: ScaledPoint = scale(moved, 2);

println(scaled);                     // {x: 2, y: 2}

let scaled2: ScaledPoint = scale(moved, 2.0);

println(scaled2);                    // {x: 2.0, y: 2.0}

// Import ciblé d'un seul alias, comme pour une classe :
//   import shapes.Point;
//
// Refusé (à décommenter pour voir l'erreur) :
//   let bad: Point = { x: 1 };       // champ 'y' manquant
//   let leak = Point;                // Point n'existe pas comme VALEUR
