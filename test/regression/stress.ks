// GC - allocation stress
for i in range(100000) {
    let a = [i, i + 1, i + 2];
    let b = {
        index: i,
        text: "stress"
    };
    let c = "object-" + str(i);
}

println("gc-stress-complete");
