// ====================================================================
// Kastel — Pauses et ordonnancement
// Notions : sleep(ms) coopératif : la tâche la plus courte finit d'abord
// Résultat attendu :
//   court
//   long
// ====================================================================

func dormir(ms: int, nom: str, journal) {
    sleep(ms);
    journal.add(nom);
}

let journal = [];
let lent = spawn(dormir, 40, "long", journal);
let rapide = spawn(dormir, 5, "court", journal);

lent.join();
rapide.join();

for nom in journal {
    println(nom);
}
