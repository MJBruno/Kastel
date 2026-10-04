// ====================================================================
// Kastel — Result : les bases
// Notions : Ok, Err, is_ok, is_err, unwrap, unwrap_or, unwrap_err
// Résultat attendu :
//   true
//   true
//   10
//   7
//   oups
//   Ok(10)
//   Err(oups)
// ====================================================================

let bon: Result<int, str> = Ok(10);
let mauvais: Result<int, str> = Err("oups");

println(bon.is_ok());
println(mauvais.is_err());
println(bon.unwrap());
println(mauvais.unwrap_or(7));
println(mauvais.unwrap_err());
println(bon);
println(mauvais);
