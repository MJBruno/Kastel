// ==================================================================
// Exemple 659 — Virements protégés par un mutex
// Catégorie : Mini-projets : algorithmes, concurrence, async
// ------------------------------------------------------------------
// Projet : deux tâches font des virements en sens inverse ; le total doit rester constant.
// ------------------------------------------------------------------
// Sortie attendue :
//   1000
//   500
// ==================================================================

let m = mutex();
let compte_a = 500;
let compte_b = 500;

func aller() {
    for i in range(100) {
        m.lock();
        compte_a -= 1;
        compte_b += 1;
        m.unlock();
    }
}

func retour() {
    for i in range(100) {
        m.lock();
        compte_b -= 1;
        compte_a += 1;
        m.unlock();
    }
}

let t1 = spawn(aller);
let t2 = spawn(retour);
t1.join();
t2.join();
println(compte_a + compte_b);
println(compte_a);
