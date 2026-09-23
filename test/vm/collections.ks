let arr = [3, 1, 4, 1, 5, 9, 2, 6];
println(arr);
arr.push(10);
println(arr);
println(arr.length);
println(arr.pop());
println(arr);
arr.sort();
println(arr);
println(arr.first());
println(arr.last());
println(arr.contains(5));
println(arr.index_of(9));

let t = (1, 2, 3);
println(t);
println(t.length);
println(t.get(1));

let d = {name: "Kastel", version: 1};
println(d.get("name"));
println(d.has("version"));
d.set("version", 2);
println(d.get("version"));
println(d.keys());
