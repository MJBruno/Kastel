let d = {a: 1};
println(d.get_or('a', 9));
println(d.get_or('b', 9));
let e = {b: 2, c: 3};
d.update(e);
println(d.length());
println(d.get('c'));
