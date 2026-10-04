// ====================================================================
// Kastel — Palindrome
// Notions : reverse, comparaison de chaînes
// Résultat attendu :
//   radar : true
//   kastel : false
// ====================================================================

func est_palindrome(mot) {
    return mot == mot.reverse();
}

for mot in ["radar", "kastel"] {
    println("{} : {}", mot, est_palindrome(mot));
}
