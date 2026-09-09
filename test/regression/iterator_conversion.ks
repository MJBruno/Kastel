// ============================================================
// LIST -> ITERATOR
// ============================================================

let numbers = [10, 20, 30];

let list_iterator = numbers.to_iterator();

println(list_iterator.has_next());
println(list_iterator.next());
println(list_iterator.next());
println(list_iterator.next());
println(list_iterator.has_next());


// ============================================================
// ITERATOR -> LIST
// ============================================================

let source_iterator = [1, 2, 3, 4, 5].to_iterator();

let restored_list = source_iterator.collect();

println(restored_list);


// ============================================================
// RANGE -> ITERATOR
// ============================================================

let range_iterator = range(5);

println(range_iterator.has_next());
println(range_iterator.next());
println(range_iterator.next());
println(range_iterator.next());


// ============================================================
// RANGE -> LIST
// ============================================================

let range_list = range(5).collect();

println(range_list);


// ============================================================
// DICT -> ITERATOR
// ============================================================

let user = {
    name: "Bruno",
    language: "Rust",
    project: "Kastel"
};

let dict_iterator = user.to_iterator();

println(dict_iterator.has_next());
println(dict_iterator.next());
println(dict_iterator.next());
println(dict_iterator.next());
println(dict_iterator.has_next());


// ============================================================
// DICT -> LIST
// ============================================================

let dict_k = user.to_iterator().collect();

println(dict_k);


// ============================================================
// STRING -> ITERATOR
// ============================================================

let text_iterator = "Kastel".to_iterator();

println(text_iterator.next());
println(text_iterator.next());
println(text_iterator.next());
println(text_iterator.next());
println(text_iterator.next());
println(text_iterator.next());
println(text_iterator.has_next());


// ============================================================
// STRING -> LIST
// ============================================================

let characters = "Kastel".to_iterator().collect();

println(characters);


// ============================================================
// ITERATOR -> ITERATOR
// ============================================================

let iterator = range(3);

let same_iterator = iterator.to_iterator();

println(same_iterator.next());
println(same_iterator.next());
println(iterator.next());


// ============================================================
// LAZY TRANSFORMATION
// ============================================================

let lazy = [1, 2, 3, 4, 5]
    .to_iterator()
    .map(x => x * 2)
    .filter(x => x > 4)
    .take(2);

println(lazy.collect());


// ============================================================
// DICT -> ITERATOR -> TRANSFORMATION -> LIST
// ============================================================

let dict_val = {
    a: 1,
    b: 2,
    c: 3
};

let transformed_keys = dict_val
    .to_iterator()
    .map(x => x.upper())
    .collect();

println(transformed_keys);


// ============================================================
// RANGE -> ITERATOR -> TRANSFORMATION -> LIST
// ============================================================

let transformed_range = range(10)
    .to_iterator()
    .filter(x => x % 2 == 0)
    .map(x => x * 10)
    .take(3)
    .collect();

println(transformed_range);


// ============================================================
// PEEK / CACHE
// ============================================================

let peek_iterator = [100, 200, 300].to_iterator();

println(peek_iterator.peek());
println(peek_iterator.peek());
println(peek_iterator.next());
println(peek_iterator.next());