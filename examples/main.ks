let sum = 0;

for i in range(10) {
    if (i == 5) {
        continue;
    }

    sum += i;
}

println(sum);