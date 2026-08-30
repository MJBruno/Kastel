function add(a, b) {
    return a + b;
}

print add(2, 3);   // 5

function factorial(n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);   // récursion supportée
}

print factorial(5);   // 120

// Une fonction sans "return" explicite se termine simplement
function saluer(nom) {
    print "Bonjour, " + nom;
}

saluer("Bruno");