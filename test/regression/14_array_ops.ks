let a = [1, 2, 3];
let b = a.copy();
b[0] = 99;
println(a[0]);
println(b[0]);
let c = a.slice(1, 3);
println(c.length);
println(c[0]);
println(a.join('-'));
