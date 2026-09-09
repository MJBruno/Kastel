let text = "  Hello World  ";

println(text.length());

println(text.get(1));
println(text.char_at(1));

println(text.contains("World"));
println(text.starts_with("  He"));
println(text.ends_with("  "));

println(text.index_of("World"));
println(text.last_index_of("l"));

println(text.slice(2, 7));
println(text.substring(2, 5));

println(text.upper());
println(text.lower());

println(text.trim());
println(text.trim_start());
println(text.trim_end());

println("hello hello".replace("hello", "hi"));
println("hello hello".replace_all("hello", "hi"));

println("a,b,c".split(","));

println("-".join(["a", "b", "c"]));

println("ab".repeat(3));

println("123".to_int());
println("3.1415".to_float());

println("".is_empty());
println("12345".is_digit());
println("abc".is_alpha());
println("abc123".is_alphanumeric());

println("hello".reverse());