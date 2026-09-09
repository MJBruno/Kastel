let a = [1, 2, 3, 4];

let doubled = a.map(fn(x) {
    return x * 2;
});

let even = a.filter(fn(x) {
    return x % 2 == 0;
});

let sum = a.reduce(fn(acc, x) {
    return acc + x;
}, 0);

let has_even = a.any(fn(x) {
    return x % 2 == 0;
});

let all_positive = a.all(fn(x) {
    return x > 0;
});