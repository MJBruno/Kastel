// GC - strings
func make_string(i) {
    return "temporary-string-" + str(i);
}

let keep = "keep-me";

for i in range(10000) {
    let temp = make_string(i);
}

println(keep);
