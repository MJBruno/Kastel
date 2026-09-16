let a = [10, 20, 30, 40];
let it = a.to_iterator();
let junk = [];
let i = 0;
while i < 200 {
    junk = [i, i + 1, i + 2, i + 3];
    i += 1;
}
println(it.next());
println(it.next());
