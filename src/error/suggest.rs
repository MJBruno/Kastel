// ================================================================
// SUGGESTIONS
// ================================================================
//
// Petit utilitaire partagé pour proposer "vouliez-vous dire X ?" quand un
// nom (variable, champ d'objet...) n'est pas trouvé. Utilisé à la fois par
// le compilateur (CompileError::UndefinedVariable) et la VM
// (RuntimeError::ObjectFieldNotFound).

/// Distance de Levenshtein classique (nombre minimal d'insertions/
/// suppressions/substitutions pour passer de `a` à `b`).
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();

    let mut previous_row: Vec<usize> = (0..=b.len()).collect();
    let mut current_row = vec![0; b.len() + 1];

    for (i, &char_a) in a.iter().enumerate() {
        current_row[0] = i + 1;

        for (j, &char_b) in b.iter().enumerate() {
            let cost = if char_a == char_b { 0 } else { 1 };

            current_row[j + 1] = (previous_row[j + 1] + 1)
                .min(current_row[j] + 1)
                .min(previous_row[j] + cost);
        }

        std::mem::swap(&mut previous_row, &mut current_row);
    }

    previous_row[b.len()]
}

/// Cherche, parmi `candidates`, le nom le plus proche de `target` — ou
/// `None` si aucun n'est assez proche pour valoir une suggestion (seuil :
/// au plus 1/3 de la longueur du nom recherché, arrondi, minimum 1).
/// Évite de proposer des suggestions absurdes entre deux noms totalement
/// différents juste parce que c'est le "moins pire" de la liste.
pub fn closest_match<'a>(target: &str, candidates: impl Iterator<Item = &'a str>) -> Option<String> {
    let threshold = (target.chars().count() / 3).max(1);

    candidates
        .map(|candidate| (candidate, levenshtein(target, candidate)))
        .filter(|(candidate, distance)| *distance > 0 && *distance <= threshold && !candidate.is_empty())
        .min_by_key(|(_, distance)| *distance)
        .map(|(candidate, _)| candidate.to_string())
}