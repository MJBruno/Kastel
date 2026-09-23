let total = 0;
for i in range(1, 6) {
    total = total + i;
}
println(total);

let n = 7;
if n % 2 == 0 {
    println("even");
} else {
    println("odd");
}

let i = 0;
while i < 3 {
    println(i);
    i = i + 1;
}

for j in range(0, 5) {
    if j == 3 {
        break;
    }
    if j % 2 == 0 {
        continue;
    }
    println(j);
}
