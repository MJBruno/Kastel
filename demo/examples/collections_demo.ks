// examples/collections_demo.ks
//
// API STANDARD des collections : une seule convention, prévisible.
// Voir docs/collections-api.md.
// Lancer avec : kastel examples/collections_demo.ks

let a = [1, 2, 3];
let d = {"a": 10, "b": 20};
let t = (1, 2, 3);
let s = Set(1, 2, 3);
let texte = "Hello";

// -- size() et is_empty() : partout -------------------------------------------
println(a.size());                      // 3
println(d.size());                      // 2
println(t.size());                      // 3
println(s.size());                      // 3
println(texte.size());                  // 5
println("é".size());                    // 1  (caractères Unicode, pas octets)
println(range(0, 10, 2).to_list().size());        // 5

println(a.is_empty());                  // false
println([].is_empty());                 // true
println(range(0).to_list().is_empty());           // true

// -- contains() ----------------------------------------------------------------------
println(a.contains(2));                 // true
println(t.contains(9));                 // false
println(s.contains(3));                 // true
println(texte.contains("ll"));          // true
println(d.contains("a"));               // true  (pour un dict : la CLÉ)

// -- List : add() / remove() ------------------------------------------------------
a.add(4);                               // [1, 2, 3, 4]
a.remove(2);                            // retire la VALEUR 2 -> [1, 3, 4]

println(a);                             // [1, 3, 4]
println(a.remove_at(0));                // 1  (retrait par POSITION)
println(a);                             // [3, 4]

// -- copy() : collections mutables seulement ------------------------------------
let b = a.copy();

b.add(99);

println(a.contains(99));                // false
println(b.contains(99));                // true

// -- Dict ---------------------------------------------------------------------------------
d.set("c", 30);

println(d.get("c"));                    // 30
println(d["c"]);                        // 30  (l'indexation reste la forme naturelle)
println(d.keys());                      // ["a", "b", "c"]
println(d.values());                    // [10, 20, 30]
println(d.entries());                   // [["a", 10], ["b", 20], ["c", 30]]

// -- Tuple : volontairement minimal -----------------------------------------------
println(t.first());                     // 1
println(t.last());                      // 3
println(t.get(1));                      // 2
println(t.index_of(3));                 // 2
println(t.to_list());                   // [1, 2, 3]

// -- Range ---------------------------------------------------------------------------------
let r = range(2, 10, 2);

// println(r.start());                     // 2
// println(r.stop());                      // 10
// println(r.step());                      // 2

// -- clear() : collections mutables seulement -----------------------------------
a.clear();
d.clear();
s.clear();

println(a.size() + d.size() + s.size());   // 0

// -- to_string() ---------------------------------------------------------------------------
println(t.to_string());                 // (1, 2, 3)
println(b.to_string());                 // [3, 4, 99]

// -- iter() : la syntaxe principale reste `for x in ...` ----------------------
let it = b.iter();

println(it.next());                     // 3

for x in b {
    println(x);                         // 3, 4, 99
}

// Supprimés (erreur avec « Vouliez-vous dire ... ») :
//   length()  -> size()      push(x)  -> add(x)
//   d.has(k)  -> d.contains(k)         d.items()  -> d.entries()
//   x.to_iterator() -> x.iter()
