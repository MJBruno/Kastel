// std/collections.ks
//
// Plusieurs méthodes natives de List/Dict utilisent une sentinelle
// plutôt qu'un Option pour signaler une absence : index_of() renvoie
// -1, first()/last()/pop() renvoient le `None` brut (pas un
// Object::Option) sur une liste vide. Ce module remonte tout ça vers
// des Option<T>/Result<T, E> explicites, et ajoute quelques
// combinateurs génériques construits avec `match`.

// ------------------------------------------------------------------
// List<T>
// ------------------------------------------------------------------

export func first_opt<T>(values: List<T>) -> Option<T> {
    if values.is_empty() {
        return None;
    }
    return Some(values.first());
}

export func last_opt<T>(values: List<T>) -> Option<T> {
    if values.is_empty() {
        return None;
    }
    return Some(values.last());
}

// Retire et renvoie le dernier élément, ou None si la liste est déjà
// vide (au lieu du `None` brut ambigu de pop() natif).
export func pop_opt<T>(values: List<T>) -> Option<T> {
    if values.is_empty() {
        return None;
    }
    return Some(values.pop());
}

export func get_opt<T>(values: List<T>, index: int) -> Option<T> {
    if index < 0 || index >= values.size() {
        return None;
    }
    return Some(values[index]);
}

export func index_of_opt<T>(values: List<T>, target: T) -> Option<int> {
    let index = values.index_of(target);
    if index < 0 {
        return None;
    }
    return Some(index);
}

export func find<T>(values: List<T>, predicate) -> Option<T> {
    for value in values {
        if predicate(value) {
            return Some(value);
        }
    }
    return None;
}

export func find_index<T>(values: List<T>, predicate) -> Option<int> {
    let index = 0;
    for value in values {
        if predicate(value) {
            return Some(index);
        }
        index = index + 1;
    }
    return None;
}

// Applique `transform` (T -> Result<U, E>) à chaque élément dans
// l'ordre, s'arrête à la première Err rencontrée, sinon renvoie
// Ok(liste complète des résultats).
export func try_map<T, U, E>(values: List<T>, transform) -> Result<List<U>, E> {
    let results = [];
    for value in values {
        match transform(value) {
            Ok(mapped) => {
                results.add(mapped);
            }
            Err(error) => {
                return Err(error);
            }
        }
    }
    return Ok(results);
}

// Sépare `values` en (éléments qui valident predicate, les autres).
export func partition<T>(values: List<T>, predicate) {
    let matched = [];
    let rejected = [];
    for value in values {
        if predicate(value) {
            matched.add(value);
        } else {
            rejected.add(value);
        }
    }
    return (matched, rejected);
}

// Ne garde que les Some(...) d'une liste d'Option<T>, en ignorant les
// None — l'équivalent de `filter_map(|x| x)` d'autres langages.
export func flatten_options<T>(values: List<Option<T>>) -> List<T> {
    let results = [];
    for value in values {
        match value {
            Some(inner) => {
                results.add(inner);
            }
            None => {
                // ignoré volontairement
            }
        }
    }
    return results;
}

// ------------------------------------------------------------------
// Dict<K, V>
// ------------------------------------------------------------------

// dict.get(key) natif lève une exception sur une clé absente : cette
// version passe par contains() d'abord et renvoie Option<V>.
export func try_get<K, V>(entries: Dict<K, V>, key: K) -> Option<V> {
    if entries.contains(key) {
        return Some(entries.get(key));
    }
    return None;
}

export func try_remove<K, V>(entries: Dict<K, V>, key: K) -> Option<V> {
    if entries.contains(key) {
        let value = entries.get(key);
        entries.remove(key);
        return Some(value);
    }
    return None;
}

// ------------------------------------------------------------------
// Ajouts : combinateurs generiques supplementaires
// ------------------------------------------------------------------

// Associe les elements de deux listes position par position ; s'arrete
// a la plus courte. Renvoie une List de Tuple (a, b).
export func zip<A, B>(left: List<A>, right: List<B>) {
    let pairs = [];
    let limit = left.size();
    if right.size() < limit {
        limit = right.size();
    }
    let i = 0;
    while i < limit {
        pairs.add((left[i], right[i]));
        i = i + 1;
    }
    return pairs;
}

// map puis aplatit d'un niveau : `transform` renvoie une List par element.
export func flat_map<T, U>(values: List<T>, transform) -> List<U> {
    let results = [];
    for value in values {
        for item in transform(value) {
            results.add(item);
        }
    }
    return results;
}

// Decoupe en morceaux de `size` elements (le dernier peut etre plus
// court). Err si size <= 0.
export func chunks<T>(values: List<T>, size: int) -> Result<List<List<T>>, str> {
    if size <= 0 {
        return Err("chunks: la taille doit etre strictement positive");
    }
    let result = [];
    let current = [];
    for value in values {
        current.add(value);
        if current.size() == size {
            result.add(current);
            current = [];
        }
    }
    if !current.is_empty() {
        result.add(current);
    }
    return Ok(result);
}

// Regroupe par cle : Dict<K, List<T>>. `key_of` : T -> K.
export func group_by<T, K>(values: List<T>, key_of) -> Dict<K, List<T>> {
    let groups = {};
    for value in values {
        let key = key_of(value);
        match try_get(groups, key) {
            Some(bucket) => {
                bucket.add(value);
            }
            None => {
                groups.set(key, [value]);
            }
        }
    }
    return groups;
}

// Compte les occurrences de chaque valeur : Dict<T, int>.
export func count_by<T>(values: List<T>) -> Dict<T, int> {
    let counts = {};
    for value in values {
        match try_get(counts, value) {
            Some(current) => {
                counts.set(value, current + 1);
            }
            None => {
                counts.set(value, 1);
            }
        }
    }
    return counts;
}
