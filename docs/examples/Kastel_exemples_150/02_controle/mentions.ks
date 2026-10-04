// ====================================================================
// Kastel — Chaîne de conditions
// Notions : else if en cascade dans une fonction
// Résultat attendu :
//   17 -> Très bien
//   15 -> Bien
//   12 -> Passable
//   8 -> Insuffisant
// ====================================================================

func mention(note) {
    if note >= 16 {
        return "Très bien";
    } else if note >= 14 {
        return "Bien";
    } else if note >= 10 {
        return "Passable";
    }
    return "Insuffisant";
}

for note in [17, 15, 12, 8] {
    println("{} -> {}", note, mention(note));
}
