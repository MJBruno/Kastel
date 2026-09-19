
## THE USUAL METHODS FOR STRING

```javascript
let s = "Bonjour Monde";
println(s.length()); // 13
println(s.get(2)); // n
println(s.contains('Monde')); // true
println(s.starts_with('Bon')); // true
println(s.ends_with('de')); // true
println(s.upper()); // BONJOUR MONDE
println(s.lower()); // bonjour monde
println(s.trim()); // Bonjour Monde
println(s.replace('Monde', 'Kastel')); // Bonjour Kastel
println(s.replace_all('o', '0')); // B0nj0ur M0nde  
println(s.index_of('bc')); // -1
println(s.last_index_of('bc')); // -1
println(s.slice(1, 4)); // onj
println(s.substring(2, 5)); // njo
println(s.reverse()); // ednoM ruojnoB
println(s.repeat(2)); // Bonjour MondeBonjour Monde
println(s.char_at(3)); // j
println(s.is_empty()); // false
println('123'.is_digit()); // true
println('abc'.is_alpha()); // true
println('a1'.is_alphanumeric()); // true
println('42'.to_int()); // 42
println('3.5'.to_float()); // 3.5
println(str(123)); // "123"

let c = s.to_iterator().collect() // Collects the characters of the string into a collection (like an array or list)

println(c); // ['B', 'o', 'n', 'j', 'o', 'u', 'r', ' ', 'M', 'o', 'n', 'd', 'e']
```