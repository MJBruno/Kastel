let valeur = 300;

match valeur {
    0 => println("zéro"),
    1 | 2 => println("un ou deux"),
    3..10 => println("entre 3 et 9"),
    3..=10 => println("entre 3 et 10 inclus"),
    n if n > 100 => println("grand nombre"),
    [a, b, _] => println("tableau à 3 éléments"),
    _ => println("autre"),
}

// Les branches peuvent aussi être des blocs
match valeur {
    3 => {
        println("trois");
        println("bloc complet");
    }
    _ => println("autre"),
}