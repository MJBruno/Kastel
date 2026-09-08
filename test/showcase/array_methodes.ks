println("TEST ARRAY METHODS");

let values = [1, 2, 3];

println(values.push(4));
println(values);

println(values.pop());
println(values);

values.insert(1, 99);
println(values);

println(values.remove(1));
println(values);

println(values.contains(3));
println(values.contains(100));

values.clear();
println(values);