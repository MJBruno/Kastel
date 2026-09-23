// À placer dans test/regression/types_and_aliases.ks
//
// ATTENTION : ce fichier reproduit le bug confirmé par l'audit
// (src/compiler/types.rs, Type::is_assignable_to n'a pas de cas
// Record <-> Named pour un alias qui pointe vers un record). Tant que
// le correctif n'est pas appliqué, ce fichier NE COMPILE PAS — c'est
// attendu. Une fois corrigé, il doit s'exécuter et produire la sortie
// indiquée en commentaire.

type Point = { x: int, y: int };

func origin() -> Point {
    return { x: 0, y: 0 };
}

let p: Point = origin();   // <- ligne qui échoue tant que le bug n'est pas corrigé
print(p.x);                 // 0
print(p.y);                 // 0

// Sens inverse : une valeur typée par l'alias doit aussi convenir à la
// forme record littérale équivalente.
let shape: { x: int, y: int } = origin();
print(shape.x);             // 0

// Union de types, y compris en paramètre de fonction.
type Number = int | float;

func double(n: Number) -> Number {
    return n * 2;
}

print(double(3));           // 6
print(double(2.5));         // 5.0
