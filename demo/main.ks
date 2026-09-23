func make_garbage(n) {
    let junk = [];
    let i = 0;
    while i < n {
        junk = [i, i * 2, i * i];
        i = i + 1;
    }
    return junk;
}

let total = 0;
let i = 0;
while i < 50000 {
    let temp = [i, i + 1, i + 2];
    total = total + temp.get(0);
    i = i + 1;
}

println(total);

let last = make_garbage(2000);
println(last.size());
println(last.get(0));
println(last.get(2));
