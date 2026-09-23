// examples/records_demo.ks
//
// Record (champs nommés), Dict (clés chaînes), alias de type et unions.
// Lancer avec : kastel examples/records_demo.ks

// -- Alias de type ----------------------------------------------------------------
type Person = { name: str, age: int };
type Number = int | float;

// -- Record : `{ champ: valeur }` (clés = identifiants) -----------------------
let p: Person = { name: "Bruno", age: 25 };

println(p.name);                        // Bruno
p.age = 26;                             // on modifie un champ EXISTANT
println(p);                             // {name: "Bruno", age: 26}

// La forme est fixe : ni ajout, ni retrait de champ (pour cela : Dict).
//   p.email = "x";                     // Erreur : champ inexistant

// Typage structurel : tout record qui a AU MOINS ces champs convient.
func describe(who: Person) -> str {
    return who.name + " (" + str(who.age) + ")";
}

let wider = { name: "Alice", age: 30, city: "Paris" };

println(describe(p));                   // Bruno (26)
println(describe(wider));               // Alice (30)

// Un champ peut contenir une fonction.
let counter = { count: 0, label: () => { return "compteur"; } };

println(counter.label());               // compteur

// Introspection.
println(p.keys());                      // ["name", "age"]
println(p.values());                    // ["Bruno", 26]

let copy = p.copy();                    // `let b = p;` partagerait le record

copy.age = 99;

println(p.age);                         // 26

// -- Dict : `{ "clé": valeur }` (clés = chaînes) ------------------------------------
let users: Dict<str, int> = { "bruno": 25, "alice": 30 };

println(users["bruno"]);                // 25

users["carol"] = 41;                    // ajout dynamique

println(users.size());                  // 3
println(users.contains("alice"));       // true

// On lit un dict PAR CLÉ, pas par point :
//   users.bruno                        // Erreur : vouliez-vous dire '["bruno"]' ?

// -- Union : `int | float` ---------------------------------------------------------------
func half(x: Number) -> float {
    return x / 2;
}

println(half(3));                       // 1.5
println(half(4.0));                     // 2.0

let id: str | int = "abc-1";            // n'importe quelle union

println(id);                            // abc-1

// -- List (ex-Array) ------------------------------------------------------------------------
let people: List<Person> = [p, wider];

for person in people {
    println(person.name);               // Bruno puis Alice
}
