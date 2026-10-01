// ==================================================================
// Exemple 646 — Pipeline à trois étages
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : générer, élever au carré, puis additionner les carrés pairs.
// ------------------------------------------------------------------
// Sortie attendue :
//   56
// ==================================================================

let c1 = channel<int>();
let c2 = channel<int>();

func generer() {
    for i in range(1, 7) {
        c1.send(i);
    }
}

func carre() {
    for i in range(6) {
        let n = c1.recv();
        c2.send(n * n);
    }
}

func filtrer() -> int {
    let total = 0;
    for i in range(6) {
        let v = c2.recv();
        if v % 2 == 0 {
            total += v;
        }
    }
    return total;
}

spawn(generer);
spawn(carre);
println(spawn(filtrer).join());
