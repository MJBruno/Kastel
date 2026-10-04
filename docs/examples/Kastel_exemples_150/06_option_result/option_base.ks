// ====================================================================
// Kastel — Option : les bases
// Notions : Some, None, is_some, is_none, unwrap, unwrap_or
// Résultat attendu :
//   true
//   true
//   5
//   0
//   Some(5)
//   None
// ====================================================================

let a = Some(5);
let b: Option<int> = None;

println(a.is_some());
println(b.is_none());
println(a.unwrap());
println(b.unwrap_or(0));
println(a);
println(b);
