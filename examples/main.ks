let d = dict();

d.set(1, "one");
d.set("name", "Bruno");

println(d.get(1));
println(d.get("name"));

println(d.has(1));

println(d.keys());
println(d.values());
println(d.items());

println(d[1]);

d[2] = "two";

d.remove(1);
d.clear();