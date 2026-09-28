// std/statistic.ks
//
// Statistiques descriptives de base sur des List<float>. Les mesures
// qui n'ont pas de sens sur un jeu de données vide (mean, median, ...)
// renvoient Option<float> ; celles qui comparent deux séries
// (covariance, correlation, linear_regression) renvoient Result<T, str>
// car l'erreur (tailles différentes, échantillon trop petit, variance
// nulle) mérite un message explicite plutôt qu'un simple None.

func sorted_copy(values: List<float>) -> List<float> {
    let copy = values.copy();
    copy.sort();
    return copy;
}

export func mean(values: List<float>) -> Option<float> {
    if values.is_empty() {
        return None;
    }

    let total = 0.0;
    for value in values {
        total = total + value;
    }
    return Some(total / values.size());
}

export func median(values: List<float>) -> Option<float> {
    if values.is_empty() {
        return None;
    }

    let sorted = sorted_copy(values);
    let count = sorted.size();
    let middle = idiv(count, 2);

    if count % 2 == 1 {
        return Some(sorted[middle]);
    }

    return Some((sorted[middle - 1] + sorted[middle]) / 2.0);
}

// Valeur la plus fréquente. En cas d'égalité, la première rencontrée
// dans `values` qui atteint le compte maximal. Générique sur T: Eq
// (pas seulement les nombres) : le mode a du sens pour tout type
// comparable.
export func mode<T: Eq>(values: List<T>) -> Option<T> {
    if values.is_empty() {
        return None;
    }

    let best_value = values.first();
    let best_count = 0;

    for candidate in values {
        let count = 0;
        for value in values {
            if value == candidate {
                count = count + 1;
            }
        }
        if count > best_count {
            best_count = count;
            best_value = candidate;
        }
    }

    return Some(best_value);
}

export func variance_population(values: List<float>) -> Option<float> {
    match mean(values) {
        Some(average) => {
            let total = 0.0;
            for value in values {
                let diff = value - average;
                total = total + diff * diff;
            }
            return Some(total / values.size());
        }
        None => {
            return None;
        }
    }
}

// Variance corrigée (dénominateur n - 1, correction de Bessel) : sans
// objet pour moins de 2 valeurs.
export func variance_sample(values: List<float>) -> Option<float> {
    if values.size() < 2 {
        return None;
    }

    match mean(values) {
        Some(average) => {
            let total = 0.0;
            for value in values {
                let diff = value - average;
                total = total + diff * diff;
            }
            return Some(total / (values.size() - 1));
        }
        None => {
            return None;
        }
    }
}

export func stdev_population(values: List<float>) -> Option<float> {
    match variance_population(values) {
        Some(variance) => {
            return Some(sqrt(variance));
        }
        None => {
            return None;
        }
    }
}

export func stdev_sample(values: List<float>) -> Option<float> {
    match variance_sample(values) {
        Some(variance) => {
            return Some(sqrt(variance));
        }
        None => {
            return None;
        }
    }
}

export func range_of(values: List<float>) -> Option<float> {
    if values.is_empty() {
        return None;
    }

    let smallest = values.first();
    let largest = values.first();
    for value in values {
        if value < smallest {
            smallest = value;
        }
        if value > largest {
            largest = value;
        }
    }
    return Some(largest - smallest);
}

// Percentile par interpolation linéaire entre les deux rangs les plus
// proches (méthode la plus courante : Excel PERCENTILE.INC, numpy par
// défaut). `p` entre 0.0 et 100.0.
export func percentile(values: List<float>, p: float) -> Option<float> {
    if values.is_empty() {
        return None;
    }
    if p < 0.0 || p > 100.0 {
        return None;
    }

    let sorted = sorted_copy(values);
    let count = sorted.size();

    if count == 1 {
        return Some(sorted[0]);
    }

    let rank = (p / 100.0) * (count - 1);
    let lower = floor(rank);
    let upper = ceil(rank);

    if lower == upper {
        return Some(sorted[lower]);
    }

    let fraction = rank - lower;
    return Some(sorted[lower] + (sorted[upper] - sorted[lower]) * fraction);
}

export func covariance(xs: List<float>, ys: List<float>) -> Result<float, str> {
    if xs.size() != ys.size() {
        return Err("covariance: les deux series doivent avoir la meme taille");
    }
    if xs.size() < 2 {
        return Err("covariance: au moins 2 valeurs sont necessaires");
    }

    let mean_x = mean(xs).unwrap();
    let mean_y = mean(ys).unwrap();

    let total = 0.0;
    let i = 0;
    while i < xs.size() {
        total = total + (xs[i] - mean_x) * (ys[i] - mean_y);
        i = i + 1;
    }

    return Ok(total / (xs.size() - 1));
}

// Coefficient de corrélation de Pearson, entre -1 (anti-corrélation
// parfaite) et 1 (corrélation parfaite).
export func correlation(xs: List<float>, ys: List<float>) -> Result<float, str> {
    match covariance(xs, ys) {
        Ok(cov) => {
            let stdev_x = stdev_sample(xs).unwrap();
            let stdev_y = stdev_sample(ys).unwrap();

            if stdev_x == 0.0 || stdev_y == 0.0 {
                return Err("correlation: variance nulle sur au moins une des deux series");
            }

            return Ok(cov / (stdev_x * stdev_y));
        }
        Err(error) => {
            return Err(error);
        }
    }
}

// Régression linéaire simple (moindres carrés) : ys ~= slope * xs +
// intercept. Renvoie (slope, intercept).
export func linear_regression(xs: List<float>, ys: List<float>) {
    if xs.size() != ys.size() {
        return Err("linear_regression: les deux series doivent avoir la meme taille");
    }
    if xs.size() < 2 {
        return Err("linear_regression: au moins 2 points sont necessaires");
    }

    let mean_x = mean(xs).unwrap();
    let mean_y = mean(ys).unwrap();

    let numerator = 0.0;
    let denominator = 0.0;
    let i = 0;
    while i < xs.size() {
        let dx = xs[i] - mean_x;
        numerator = numerator + dx * (ys[i] - mean_y);
        denominator = denominator + dx * dx;
        i = i + 1;
    }

    if denominator == 0.0 {
        return Err("linear_regression: toutes les valeurs de x sont identiques");
    }

    let slope = numerator / denominator;
    let intercept = mean_y - slope * mean_x;

    return Ok((slope, intercept));
}
