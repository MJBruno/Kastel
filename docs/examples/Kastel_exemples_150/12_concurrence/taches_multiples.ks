// ====================================================================
// Kastel — Plusieurs tâches
// Notions : spawn de plusieurs tâches puis join de chacune
// Résultat attendu :
//   a=3 b=7 c=11
// ====================================================================

func additionner(x: int, y: int) -> int {
    return x + y;
}

let a = spawn(additionner, 1, 2);
let b = spawn(additionner, 3, 4);
let c = spawn(additionner, 5, 6);

println("a={} b={} c={}", a.join(), b.join(), c.join());
