// ====================================================================
// Kastel — match sur des valeurs
// Notions : littéraux entiers et chaînes, joker _
// Résultat attendu :
//   lundi
//   week-end
//   jour inconnu
//   trois
// ====================================================================

func jour(n) {
    match n {
        1 => { println("lundi"); },
        6 | 7 => { println("week-end"); },
        _ => { println("jour inconnu"); }
    }
}

jour(1);
jour(7);
jour(42);

match "trois" {
    "un" => { println("1"); },
    "trois" => { println("trois"); },
    _ => { println("autre"); }
}
