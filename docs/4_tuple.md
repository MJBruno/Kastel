```javascript
let t = (10, 20, 30);
println(t.size()); // 3
println(t.get(1)); // 20
println(t.contains(30)); // true
println(t.index_of(20)); // 1
println(t.first()); // 10
println(t.last()); // 30
let a = t.to_list();
a[0] = 99;
println(a[0]); // 99
println(t.get(0)); // 10
let empty = ();
println(empty.size()); // 0
```