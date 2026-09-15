func classify(x) {
    match x {
        0 => return 'zero';
        1 | 2 | 3 => return 'small';
        4 .. 10 => return 'range';
        _ => return 'other';
    }
}

println(classify(0));
println(classify(2));
println(classify(7));
println(classify(99));
