
 // examples/visibility_demo.ks
//
// Visibilité des membres : `private` réserve un champ ou une méthode au
// corps de la classe. Le typage reste dynamique : seule la visibilité
// DÉCLARÉE est contrôlée (à la compilation quand le type est connu, et
// toujours à l'exécution).
// Lancer avec : kastel examples/visibility_demo.ks

from personne import Personne;

let p: Personne = new Personne(26);

println(p.age)

p.setAge(44);

let age: int = p.getAge();

println(age);                // 44
println(p.number());         // 22

// Interdit : `age` est privé dans Personne.
//
//   println(p.age);          // Erreur : le membre 'age' de la classe
//                            // 'Personne' est privé
//   p.age = 10;              // idem en écriture
