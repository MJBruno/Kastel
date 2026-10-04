// ====================================================================
// Kastel — Les types de base
// Notions : type(), littéraux de chaque type
// Résultat attendu :
//   int
//   float
//   string
//   bool
//   list
//   dict
//   record
//   set
//   tuple
//   None
// ====================================================================

println("{}", type(42));
println("{}", type(3.14));
println("{}", type("texte"));
println("{}", type(true));
println("{}", type([1, 2, 3]));
println("{}", type({"a": 1}));
println("{}", type({x: 1, y: 2}));
println("{}", type(Set(1, 2)));
println("{}", type((1, 2)));
println("{}", type(None));
