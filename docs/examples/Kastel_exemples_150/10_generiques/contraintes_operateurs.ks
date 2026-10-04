// ====================================================================
// Kastel — Contraintes génériques
// Notions : <T: Add>, <T: Ord>, <T: Add + Eq>
// Résultat attendu :
//   7
//   foobar
//   9
//   true
// ====================================================================

func additionner<T: Add>(a: T, b: T) -> T {
    return a + b;
}

func plus_grand<T: Ord>(a: T, b: T) -> T {
    return a > b ? a : b;
}

func somme_si_differents<T: Add + Eq>(a: T, b: T) -> T {
    if a == b {
        return a;
    }
    return a + b;
}

println(additionner(3, 4));
println(additionner<str>("foo", "bar"));
println(plus_grand(4, 9));
println(somme_si_differents(5, 5) == 5);
