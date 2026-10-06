# `std.datetime`

`std.datetime` fournit un type `DateTime` UTC basé sur les millisecondes depuis l'Unix epoch (`1970-01-01T00:00:00Z`).

## Précision

La précision contractuelle du module est la milliseconde :

- `millisecond()` retourne une valeur de `0` à `999`.
- `from_ymdhms_millis(...)` conserve exactement la milliseconde fournie.
- `timestamp_millis()` retourne l'horodatage en millisecondes.
- `now_millis()` convertit la fraction de seconde fournie par `clock()`.
- aucune précision microseconde ou nanoseconde n'est garantie.

## Validation

Les composantes de date et d'heure disposent de validations réutilisables :

```text
is_valid_date(year, month, day)
validate_date(year, month, day)

is_valid_time(hour, minute, second, millisecond)
validate_time(hour, minute, second, millisecond)
```

Les plages temporelles sont :

```text
hour        0..23
minute      0..59
second      0..59
millisecond 0..999
```

Pour une date civile, le mois doit être compris entre `1` et `12` et le jour respecte le nombre réel de jours du mois, y compris les années bissextiles.

## Construction

```text
match DateTime.from_ymdhms_millis(2026, 10, 6, 15, 42, 17, 235) {
    Ok(value) => println(value.to_iso_string());
    Err(error) => println(error);
}
```

## Lecture des composantes

```text
date.year()
date.month()
date.day()
date.hour()
date.minute()
date.second()
date.millisecond()
date.time_millis()
```

`time_millis()` correspond au nombre de millisecondes écoulées depuis minuit.

## Arithmétique

```text
date.add_milliseconds(250)
date.add_seconds(5)
date.add_minutes(2)
date.add_hours(1)
date.add_days(1)
```

Les différences sont signées :

```text
date.difference_millis(other)
date.difference_seconds(other)
date.difference_minutes(other)
date.difference_hours(other)
```

## Horodatage courant

```text
let current = DateTime.now();
let epoch_ms = now_millis();
```

`clock()` est une horloge civile. Elle ne constitue pas une garantie d'horloge monotone pour les benchmarks ou la mesure fine d'une durée.
