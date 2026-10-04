// ====================================================================
// Kastel — Importer des fonctions
// Notions : export func / export let, from ... import { a, b as c }
// Résultat attendu :
//   12
//   Bonjour, Zoé !
//   3.14
// ====================================================================

from outils import { doubler, saluer as dire_bonjour, PI_APPROX };

println(doubler(6));
println(dire_bonjour("Zoé"));
println(PI_APPROX);
