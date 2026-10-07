import std.collections;

let values = [10, 20, 30];

match collections.first_opt(values) {
    Some(value) => println(value);
    None => println("liste vide");
}

match collections.get_opt(values, 1) {
    Some(value) => println(value);
    None => println("index invalide");
}

let pairs = collections.zip(["a", "b"], [1, 2]);
println(pairs);

match collections.chunks([1, 2, 3, 4, 5], 2) {
    Ok(value) => println(value);
    Err(error) => println(error);
}

match collections.find(values, func(value) {
        return value > 15;
}) {
    Some(value) => println(value);
    None => println("aucun resultat");
}

let partitioned = collections.partition(values, func(value) {
        return value >= 20;
});
println(partitioned.matched);
println(partitioned.rejected);

let options = [Some(1), None, Some(3)];
println(collections.flatten_options < dynamic > (options));

println("std.collections: OK");
