// À placer dans test/regression/collections.ks
// Couvre : API standardisée size()/is_empty()/contains()/copy()/clear(),
// Dict (clés chaînes {"a":1}), Record (clés identifiants {a: 1}),
// List (ex-Array), Tuple immuable, String.size() en caractères Unicode.

let l = [1, 2, 3];
print(l.size());          // 3
l.add(4);
print(l.contains(4));     // true
print(l.is_empty());      // false

let l_ref = l;
l_ref.add(5);
print(l.size());          // 5 (référence partagée)

let l_copy = l.copy();
l_copy.add(6);
print(l.size());          // 5 (copie indépendante)

l.clear();
print(l.is_empty());      // true

let d = {"name": "bruno", "age": 25};
print(d.size());          // 2
print(d.get("name"));     // bruno
d.set("age", 26);
print(d.get("age"));      // 26
print(d.contains("name")); // true

let rec = { name: "Bruno", age: 25 };
print(rec.age);           // 25
print(rec.name);          // Bruno

// Tuple : minimal, immuable, pas de copy().
let t = (1, "a", 3.0);
print(t.size());          // 3
print(t[0]);              // 1

// Unicode : "café" fait 4 caractères, pas 5 octets.
print("café".size());     // 4
