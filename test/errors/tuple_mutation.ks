// À placer dans test/errors/tuple_mutation.ks
// Doit échouer : Tuple est immuable, l'affectation par index est interdite.

let t = (1, 2, 3);
t[0] = 99;
print(t[0]);
