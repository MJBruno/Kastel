import std.core;

println(core.is_int(42));
println(core.is_float(3.14));
println(core.is_number(42));
println(core.is_bool(true));
println(core.is_string("Kastel"));
println(core.is_list([1, 2, 3]));
println(core.is_dict({"name": "Kastel"}));
println(core.is_tuple((10, "ok")));
println(core.is_none(None));
println(core.type_name(42));

core.require(2 + 2 == 4, "arithmetique invalide");
println(core.identity("identity ok"));

core.repeat(3, func() {
    println("repeat");
});

println("std.core: OK");
