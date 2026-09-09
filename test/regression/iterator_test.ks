let it = range(5);

println(it.has_next());
println(it.peek());
println(it.peek());

println(it.next());
println(it.next());

println(it.has_next());
println(it.next());


let mapped = range(5)
    .map(x => x * 2);

println(mapped.collect());


let filtered = range(10)
    .filter(x => x % 2 == 0);

println(filtered.take(3).collect());


let skipped = range(10)
    .skip(5);

println(skipped.take(3).collect());


let pipeline = range(20)
    .skip(5)
    .filter(x => x % 2 == 0)
    .map(x => x * 10)
    .take(3);

println(pipeline.collect());


let count_iterator = range(10);

println(count_iterator.count());


let any_iterator = range(10);

println(any_iterator.any(x => x == 5));


let all_iterator = range(5);

println(all_iterator.all(x => x < 5));


let peek_iterator = range(3);

println(peek_iterator.has_next());
println(peek_iterator.peek());
println(peek_iterator.has_next());
println(peek_iterator.next());
println(peek_iterator.next());
println(peek_iterator.next());
println(peek_iterator.has_next());