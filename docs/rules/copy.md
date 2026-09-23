## COPY et REFERENCE

```javascript
//=============== REFERENCE ============
let a = [1, 2, 3];
let b = a;

b[0] = 100;

println(a[0] == 1) //false
println(a[0] == 100) //true



// =============== COPY ============

let a = [1, 2, 3];
let b = a.copy();

b[0] = 100;

println(a[0] == 1);    // true
println(b[0] == 100);  // true


//=============== FUNCTION ============

func modify(arr) {
    arr[0] = 100;
}

let a = [1, 2, 3];
modify(a);

println(a[0]); // 100
```




## COPY
```python
Primitive immutable
    → pas de copy()

Tuple immutable
    → pas de copy()

List mutable
    → copy()

Dict mutable
    → copy()

Instance
    → copy() seulement si explicitement défini
```

## Pour TUPLE:

```javascript
let a = (1, 2, 3);
let b = a.to_array();

b[0] = 100;

println(a[0] == 1);    // true
println(b[0] == 100);  // 

tuple.copy()     ❌
tuple.to_list() ✅

```