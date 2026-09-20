// examples/types_demo.ks
//
// Inférence et annotations de types.
// Lancer avec : kastel examples/types_demo.ks

let x = 10;                         // inféré: int
let y: int = 20;                    // annotation explicite
let z = x + y;                      // inféré: int

let name: str = "Bruno";

let values: Array<int> = [1, 2, 3];
let users: Dict<str, int> = {
    age: 25
};

func add(a: int, b: int) -> int {
    return a + b;
}

let result = add(10, 20);           // int

// -- Tuples typés : compatibles élément par élément ------------------------
let pair: Tuple<int, str> = (1, "a");
let wide: Tuple<float, str> = pair;   // int -> float accepté

println(z);                         // 30
println(name);                      // Bruno
println(values);                    // [1, 2, 3]
println(users["age"]);              // 25
println(result);                    // 30
println(pair[0]);                   // 1
println(wide);                      // (1, "a")
