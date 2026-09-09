let text = "Kastel";

println(text.length());
println(text.upper());
println(text.lower());

let numbers = [1, 2, 3, 4, 5];

let iterator = numbers.to_iterator();

println(iterator.next());
println(iterator.peek());
println(iterator.next());

let rest = iterator.collect();

println(rest);


let filtered = range(10)
    .filter(x => x % 2 == 0)
    .map(x => x * 10)
    .take(3)
    .collect();

println(filtered);


let data = {
    name: "Bruno",
    language: "Rust",
    project: "Kastel"
};

println(data.length());
println(data.get("name"));
println(data.has("language"));
println(data.keys());
println(data.values());
println(data.items());


data.set("version", 1);

println(data.get_or("missing", "default"));

let copy = data.copy();

copy.set("name", "Kastel");

println(data.get("name"));
println(copy.get("name"));