```javascript
let d = {'name': 'Bruno', 'age': 30}; 
println(d.size()); // 2
println(d.get('name')); // Bruno
println(d.contains('age')); // true
println(d.contains('missing')); // false
d.set('city', 'Nosy Be'); 
println(d.get('city')); // Nosy Be
d['age'] = 31;
println(d['age']); // 31
println(d.remove('city')); // Nosy Be
println(d.contains('city')); // false
let keys = d.keys(); 
let values = d.values(); 
let items = d.entries();
println(keys.size()); // 2
println(values.size()); // 2
println(items.size()); // 2
println(values[0]); // Bruno
let copy = d.copy();
copy.set('c', 3);
println(d.contains('c')); // false
println(copy.contains('c')); // true
d.clear();
println(d.size()); // 0
```