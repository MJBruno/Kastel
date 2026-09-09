let user = {
    name: "Bruno",
    age: 25,
    language: "Rust"
};

println(user.length());

println(user.get("name"));
println(user.get("age"));

println(user.has("name"));
println(user.has("missing"));

user.set("country", "Madagascar");

println(user);
println(user.length());

println(user.remove("country"));
println(user);

println(user.keys());
println(user.values());
println(user.items());

println(user.get_or("name", "Unknown"));
println(user.get_or("missing", "Unknown"));

let other = {
    editor: "VSCode",
    level: "beginner"
};

user.update(other);

println(user);

let copy = user.copy();

copy.set("name", "Kastel");

println(user);
println(copy);

user.clear();

println(user);
println(user.length());