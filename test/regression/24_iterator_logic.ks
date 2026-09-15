let any_value = range(1, 6).any(x => x == 4);
let all_value = range(1, 6).all(x => x > 0);
let filtered = range(0, 10).filter(x => x % 2 == 0).collect();
println(any_value);
println(all_value);
println(filtered.length);
println(filtered[3]);
