// À placer dans test/regression/set.ks
// Couvre : Set(...), unicité garantie, add/remove/contains/size/is_empty/clear,
// union/intersection/difference/symmetric_difference/is_subset/is_superset,
// sémantique référence + copy().

let s = Set(1, 2, 2, 3, 1);
print(s.size());          // 3
print(s.contains(2));     // true
print(s.is_empty());      // false

s.add(4);
print(s.size());          // 4
print(s.remove(4));       // true
print(s.remove(4));       // false (déjà absent, pas d'erreur)

let a = Set(1, 2, 3);
let b = Set(2, 3, 4);

print(a.union(b).size());               // 4
print(a.intersection(b).size());        // 2
print(a.difference(b).size());          // 1
print(a.symmetric_difference(b).size()); // 2
print(Set(1, 2).is_subset(a));          // true
print(a.is_superset(Set(1, 2)));        // true

// Partage par référence vs copie indépendante.
let ref_ = a;
ref_.add(99);
print(a.contains(99));    // true (même objet)

let copy_ = a.copy();
copy_.add(1000);
print(a.contains(1000));  // false (copie indépendante)

a.clear();
print(a.is_empty());      // true

// Auto-référence : ne doit pas paniquer (emprunt mutable réentrant).
let self_set = Set(1);
self_set.add(self_set);
print(self_set.size());   // 2
