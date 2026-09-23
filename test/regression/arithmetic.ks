// À placer dans test/regression/arithmetic.ks
// Couvre : entiers 64 bits signés, promotion int->float, division "vraie",
// modulo, wrapping_* explicite.

print(2 + 2);                 // 4
print(7 / 2);                 // 3.5  (division toujours flottante)
print(7 % 2);                 // 1
print(-7 % 2);                 // -1 (signe du dividende, comme Rust)
print(5 < 5.5);               // true (comparaison int/float mixable)
print(3 * 1.5);               // 4.5

// wrapping_* : arithmétique cyclique EXPLICITE (le "+" normal, lui,
// doit lever une erreur en cas de dépassement — voir errors/overflow.ks).
print(wrapping_add(9223372036854775807, 1));  // -9223372036854775808
print(wrapping_sub(-9223372036854775808, 1)); // 9223372036854775807
print(wrapping_mul(9223372036854775807, 2));  // -2

// Cas limite documenté : i64::MIN % -1 ne doit pas paniquer nativement.
print(-9223372036854775808 % -1); // 0
