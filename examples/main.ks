let i = 0;
while i < 5 {
    i += 1;
}
println(i);
let sum = 0;
for x in [1, 2, 3, 4] {
    if x == 3 {
        continue;
    }
    sum += x;
}
println(sum);