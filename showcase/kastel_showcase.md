# Kastel — panorama des fonctionnalités (mis à jour)

Ce document illustre tout ce qui fonctionne réellement dans Kastel aujourd'hui,
après les dernières sessions : `for..in`, `range()`, `print`/`println`/`input`,
concaténation stricte, objets dynamiques, et le garbage collector.

---

## 1. Variables : `let` et `const`

``` 
let x = 10;
const PI = 3.14159;

let a = 1, b = 2, c = 3;

x = 20;   // OK
PI = 3;   // Erreur de compilation : AssignmentToConstant("PI")
```

## 2. Types et littéraux

```
let n = 42;
let f = 3.14;
let s = "bonjour";
let b1 = true;
let b2 = false;
let vide = nil;
```
## 3. Conversions des types

```
println(int(3.9));        // 3   (tronqué vers zéro, pas arrondi)
println(int(-3.9));       // -3  (comme Python : truncation, pas floor)
println(int("42"));       // 42
println(int(true));       // 1

println(float("3.14"));   // 3.14
println(float(false));    // 0

let a = [1, 2, 3];

println(str(42));         // "42"
println(str(a));          // "[1, 2, 3]"
println(str({ a: 1 }));   // "{a: 1}"

println(bool(0));         // false
println(bool(""));        // false
println(bool([]));        // true   (⚠️ tableau vide equivaut à true)
println(bool(nil));       // false
```

## 4. Opérateurs

```
let somme = 2 + 3;
let diff  = 5 - 2;
let prod  = 4 * 6;
let quot  = 10 / 3;
let reste = 10 % 3;
let neg   = -x;

let eg = (a == b);
let inf = (a < b);

let et = (a > 0) && (b > 0);
let ou = (a > 0) || (b > 0);
let non = !et;

let statut = (age >= 18) ? "majeur" : "mineur";
let mention = (note >= 16) ? "très bien" : (note >= 12) ? "bien" : "passable";

// ⚠️ Concaténation STRICTE depuis la dernière session :
// seul String + String fonctionne. String + Number est désormais une
// erreur de type (avant, il y avait une coercion permissive façon JS).
let ok = "Age: " + "25";     // "Age: 25"
let ko = "Age: " + 25;       // 🚨 Erreur de type

// Pour combiner texte et nombre, utiliser format() ou print/println :
let phrase = format("Age: {}", 25);   // "Age: 25"
```

## 5. Structures de contrôle

```
if (x > 0) {
    println("positif");
} else {
    if (x < 0) {
        println("négatif");
    } else {
        println("zéro");
    }
}

let i = 0;
while (i < 5) {
    println(i);
    i = i + 1;
}

// Le for classique (style C) n'existe plus. Seul for..in subsiste :
for i in range(5) {
    println(i);            // 0 1 2 3 4
}

for i in range(2, 10, 2) {
    println(i);            // 2 4 6 8
}

for i in range(10, 0, -1) {
    if (i == 5) { break; }
    println(i);             // 10 9 8 7 6
}

for x in [10, 20, 30] {
    if (x == 20) { continue; }
    println(x);              // 10 30
}
```

## 6. Fonctions (et closures)

```
function add(a, b) {
    return a + b;
}

println(add(2, 3));   // 5

function factorial(n) {
    if (n <= 1) { return 1; }
    return n * factorial(n - 1);
}

println(factorial(5));   // 120

// Closures : une fonction imbriquée capture les variables de la fonction
// englobante (upvalues).
function make_counter() {
    let count = 0;

    function increment() {
        count = count + 1;
        return count;
    }

    return increment;
}

let counter = make_counter();
println(counter());   // 1
println(counter());   // 2
println(counter());   // 3
```

## 7. Tableaux

```
let arr = [1, 2, 3];

println(arr[0]);        // 1
arr[0] = 100;
println(arr.length);    // 3

arr.push(4);
let dernier = arr.pop();   // tableau vide -> nil, jamais d'erreur

arr.insert(1, 99);
arr.remove(0);
arr.clear();
let existe = arr.contains(5);

let matrice = [[1, 2], [3, 4]];
println(matrice[0][1]);   // 2
```

## 8. Objets (nouveau)

```
let user = {
    name: "Bruno",
    age: 25,
};

println(user.name);   // Bruno
println(user.age);    // 25

user.age = 26;               // affectation de champ existant
user.city = "Antananarivo";  // ajout dynamique d'un nouveau champ

println(user);   // {name: Bruno, age: 26, city: Antananarivo}

// Les objets peuvent contenir n'importe quelle valeur, y compris
// d'autres objets ou des tableaux :
let config = {
    debug: true,
    limits: { max: 100, min: 0 },
    tags: ["a", "b", "c"],
};

println(config.limits.max);   // 100
println(config.tags[1]);      // b
```

## 9. Affichage, saisie, formatage

```
print("Sans saut de ligne");
println(" — puis avec un saut de ligne");

println("Bonjour {}, tu as {} ans", "Alice", 30);
let phrase = format("Score: {}/{}", 8, 10);   // ne s'affiche pas, juste construit

let nom = input("Quel est ton nom ? ");
println("Bonjour " + nom);
```

## 10. Modules : `import` / `from ... import` / `export`

`math.ks` :
```
export function square(x) { return x * x; }
export const PI = 3.14159;
```

`main.ks` :
```
import math;
println(math.square(5));

from math import square as carre;
println(carre(4));
```

## 11. Fonctions natives disponibles

`clock()`, `range(stop|start,stop|start,stop,step)`, `print(...)`, `println(...)`,
`format(...)`, `input([prompt])`, `push`, `pop`, `length`, `insert`, `remove`.

---

## Le garbage collector : cycles + `trace_gc`

Kastel utilise `Rc` (comptage de références) pour l'essentiel, complété par un
**collecteur de cycles** (mark & sweep) qui tourne automatiquement entre deux
instructions dès qu'un seuil d'allocations est dépassé — aucune action requise
côté script.

### Exemple de cycle (tableaux, closures, objets peuvent tous en former un)

```
function make_cycle() {
    let arr = [];

    function grab() {
        return arr;
    }

    arr.push(grab);   // arr -> grab -> upvalue fermée -> arr : cycle
}

// Cycle via un objet auto-référentiel :
function make_self_ref() {
    let obj = { name: "boucle" };
    obj.self = obj;    // obj -> obj : cycle direct
}

for i in range(20000) {
    make_cycle();
    make_self_ref();
}

println("Terminé sans fuite mémoire non bornée.");
```

### Observer le collecteur avec `KASTEL_TRACE_GC`

Aucun changement de code requis — juste une variable d'environnement au lancement :

```
KASTEL_TRACE_GC=1 ./kastel gc_stress_test.ks
```

Sortie attendue (sur stderr), à chaque passage de collecte :

```
-- gc begin
   mark: 3 tableaux, 2 closures, 2 upvalues, 1 objets atteignables
   sweep: tableaux 130 -> 45 (85 cycles cassés)
          closures 90 -> 12 (78 cycles cassés)
          upvalues 90 -> 12 (78 cycles cassés)
          objets   64 -> 3 (61 cycles cassés)
-- gc end (total: 302 cycles cassés, 0.412ms, prochain seuil: 256)
```

Ça permet de vérifier concrètement que :
- le nombre d'objets **vivants** (`X -> Y`) redescend après chaque passage,
- le **seuil** s'adapte à la taille du tas réellement vivant plutôt que de
  doubler aveuglément,
- le coût d'un passage reste de l'ordre de la fraction de milliseconde même
  avec des dizaines de milliers d'objets trackés.

---

## Ce qui n'est toujours pas supporté

| Fonctionnalité | Statut |
|---|---|
| Fonctions anonymes / lambdas (`let f = function(x) {...};`) | AST/parser à étendre (VM déjà prête) |
| `else if` sans accolades imbriquées | Sucre syntaxique non implémenté |
| Opérateurs composés (`+=`, `-=`, etc.) | Non implémentés |
| `for key in obj` (itération sur les clés d'un objet) | `for..in` ne parcourt que des tableaux actuellement |
| Suppression de champ (`delete obj.x`) | Non implémenté |
| REPL avec état persistant entre les lignes | Chaque ligne repart de zéro |
