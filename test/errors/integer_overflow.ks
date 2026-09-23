// À placer dans test/errors/integer_overflow.ks
// Doit échouer : addition qui dépasse i64::MAX (pas de wrap silencieux).

let x = 9223372036854775807 + 1;
print(x);
