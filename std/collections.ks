// std/collections.ks
//
// Utilitaires de haut niveau sur les tableaux, écrits en Kastel
// au-dessus des méthodes natives d'array (push, get, length, ...).
//
// Usage : from std.collections import enumerate, zip, flatten;



export func enumerate(items) {
    let result = [];
    let i = 0;

    while i < items.size() {
        result.add([i, items.get(i)]);
        i = i + 1;
    }

    return result;
}

export func zip(a, b) {
    let result = [];
    let length = a.size();

    if b.size() < length {
        length = b.size();
    }

    let i = 0;

    while i < length {
        result.add([a.get(i), b.get(i)]);
        i = i + 1;
    }

    return result;
}

// Aplatit un tableau de profondeur arbitraire en un tableau plat.
export func flatten(items) {
    let result = [];

    for item in items {
        if type(item) == "list" {
            for inner in flatten(item) {
                result.add(inner);
            }
        } else {
            result.add(item);
        }
    }

    return result;
}

// Nouveau tableau sans doublons, dans l'ordre de première apparition.
export func unique(items) {
    let result = [];

    for item in items {
        if !result.contains(item) {
            result.add(item);
        }
    }

    return result;
}

// Découpe `items` en sous-tableaux d'au plus `size` éléments.
export func chunk(items, size) {
    let result = [];
    let current = [];

    for item in items {
        current.add(item);

        if current.size() == size {
            result.add(current);
            current = [];
        }
    }

    if current.size() > 0 {
        result.add(current);
    }

    return result;
}

// Équivalent de `list(range(stop))` : un tableau [0, 1, ..., stop-1].
export func range_array(stop) {
    let result = [];

    for i in range(stop) {
        result.add(i);
    }

    return result;
}
