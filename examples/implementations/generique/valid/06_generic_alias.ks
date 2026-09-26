// Alias générique avec deux paramètres.
type Pair<A, B> = { first: A, second: B };

type IntPair = Pair<int, str>;

let pair: IntPair = {
    first: 7,
    second: "seven"
};

let first: int = pair.first;
let second: str = pair.second;

println(first);
println(second);
