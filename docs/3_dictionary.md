```javascript

let d = {name: 'Bruno', age: 30}; 
println(d.length()); // 2
println(d.get('name')); // Bruno
println(d.has('age')); // true
println(d.has('missing')); // false
d.set('city', 'Nosy Be'); 
println(d.get('city')); // Nosy Be
d['age'] = 31;
println(d['age']); // 31
println(d.remove('city')); // Nosy Be
println(d.has('city')); // false
let keys = d.keys(); 
let values = d.values(); 
let items = d.items();
println(keys.length); // 2
println(values.length); // 2
println(items.length); // 2
println(values[0]); // Bruno
let copy = d.copy();
copy.set('c', 3);
println(d.has('c')); // false
println(copy.has('c')); // true
d.clear();
println(d.length()); // 0
```