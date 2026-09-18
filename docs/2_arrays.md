```javascript  
let a = [10, 30, 20]; 
println(a.length); // 3
println(a[0]); // 10
a[1] = 25; 
println(a[1]); // 25
println(a.contains(25)); // true
println(a.index_of(30)); // 2
println(a.first()); // 10
println(a.last()); // 30
println(a.pop()); // 30
a.push(40);
println(a.last()); // 40
a.insert(1, 9);
println(a.remove(1));
a.reverse();
println(a)
a.sort();
println(a)
let b = a.copy();
b[0] = 99;
println(a[0]);
println(b[0]);
let c = a.slice(1, 3);
println(c.length);
println(c[0]);
println(a.join('-'));
a.clear();
println(a)
```
