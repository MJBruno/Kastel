// std/datetime.ks
//
// Dates et heures, en secondes depuis l'epoch Unix (1970-01-01
// 00:00:00 UTC) — comme `clock()`, la native dont ce module part.
//
// Usage :
//
//   from std.datetime import DateTime, now, from_timestamp;
//
//   let birth = new DateTime(1995, 6, 21, 8, 30, 0);
//   println(birth.to_string());        // 1995-06-21 08:30:00
//   println(birth.weekday_name());     // Mercredi
//
//   let today = now();
//   println(today.difference_days(birth));
//
// Limite connue : les méthodes de décomposition (year/month/day/...)
// sont garanties correctes pour les dates >= 1970-01-01. Avant
// l'epoch, `timestamp()` reste exact (utile pour des durées), mais
// l'affichage calendaire n'est pas garanti.

const WEEKDAY_NAMES = ["Dimanche", "Lundi", "Mardi", "Mercredi", "Jeudi", "Vendredi", "Samedi"];
const MONTH_NAMES = ["Janvier", "Février", "Mars", "Avril", "Mai", "Juin", "Juillet", "Août",
    "Septembre", "Octobre", "Novembre", "Décembre"];

func pad2(n) {
    if n < 10 {
        return format("0{}", n);
    }

    return str(n);
}

export class DateTime {
    func init(year, month, day, hour, minute, second) {
        let days = this.days_before_year(year);
        let m = 1;

        while m < month {
            days = days + this.days_in_month(year, m);
            m = m + 1;
        }

        days = days + (day - 1);

        this.value = days * 86400 + hour * 3600 + minute * 60 + second;
    }

    func timestamp() {
        return this.value;
    }

    // ========================================================
    // Calendrier
    // ========================================================

    func is_leap_year(year) {
        if year % 400 == 0 {
            return true;
        }

        if year % 100 == 0 {
            return false;
        }

        return year % 4 == 0;
    }

    func days_in_year(year) {
        if this.is_leap_year(year) {
            return 366;
        }

        return 365;
    }

    func days_in_month(year, month) {
        match month {
            1 => return 31;

            2 => {
                if this.is_leap_year(year) {
                    return 29;
                }

                return 28;
            }

            3 => return 31;
            4 => return 30;
            5 => return 31;
            6 => return 30;
            7 => return 31;
            8 => return 31;
            9 => return 30;
            10 => return 31;
            11 => return 30;
            12 => return 31;

            _ => throw format("days_in_month: mois invalide ({})", month);
        }
    }

    // Nombre de jours entre 1970-01-01 et le 1er janvier de `year`
    // (négatif si `year` < 1970).
    func days_before_year(year) {
        let days = 0;

        if year >= 1970 {
            let y = 1970;

            while y < year {
                days = days + this.days_in_year(y);
                y = y + 1;
            }
        } else {
            let y = year;

            while y < 1970 {
                days = days - this.days_in_year(y);
                y = y + 1;
            }
        }

        return days;
    }

    // ========================================================
    // Décomposition — un seul passage, réutilisé par
    // year()/month()/day() pour éviter de recalculer trois fois la
    // même boucle.
    // ========================================================

    func ymd() {
        let days = floor(this.value / 86400);
        let year = 1970;

        while days >= this.days_in_year(year) {
            days = days - this.days_in_year(year);
            year = year + 1;
        }

        let month = 1;

        while days >= this.days_in_month(year, month) {
            days = days - this.days_in_month(year, month);
            month = month + 1;
        }

        return [year, month, days + 1];
    }

    func year() {
        return this.ymd().get(0);
    }

    func month() {
        return this.ymd().get(1);
    }

    func day() {
        return this.ymd().get(2);
    }

    func hour() {
        return floor((this.value % 86400) / 3600);
    }

    func minute() {
        return floor((this.value % 3600) / 60);
    }

    func second() {
        return floor(this.value % 60);
    }

    // 0 = dimanche ... 6 = samedi (1970-01-01 était un jeudi).
    func weekday() {
        let days = floor(this.value / 86400);

        return ((days % 7) + 4 + 7) % 7;
    }

    func weekday_name() {
        return WEEKDAY_NAMES.get(this.weekday());
    }

    func month_name() {
        return MONTH_NAMES.get(this.month() - 1);
    }

    // ========================================================
    // Formatage
    // ========================================================

    func date() {
        let parts = this.ymd();

        return format("{}-{}-{}", parts.get(0), pad2(parts.get(1)), pad2(parts.get(2)));
    }

    func time() {
        return format("{}:{}:{}", pad2(this.hour()), pad2(this.minute()), pad2(this.second()));
    }

    func to_string() {
        return format("{} {}", this.date(), this.time());
    }

    // ========================================================
    // Comparaisons
    // ========================================================

    func equals(other) {
        return this.value == other.value;
    }

    func is_before(other) {
        return this.value < other.value;
    }

    func is_after(other) {
        return this.value > other.value;
    }

    // ========================================================
    // Différences
    // ========================================================

    func difference_seconds(other) {
        return this.value - other.value;
    }

    func difference_days(other) {
        return floor(this.difference_seconds(other) / 86400);
    }

    // ========================================================
    // Arithmétique — chaque add_* renvoie une NOUVELLE instance,
    // `this` n'est jamais modifié.
    // ========================================================

    func add_seconds(value) {
        return from_timestamp(this.value + value);
    }

    func add_minutes(value) {
        return this.add_seconds(value * 60);
    }

    func add_hours(value) {
        return this.add_seconds(value * 3600);
    }

    func add_days(value) {
        return this.add_seconds(value * 86400);
    }
}

// ------------------------------------------------------------------
// Constructeurs libres
//
// `DateTime.init` exige les 6 composants (année, mois, jour, heure,
// minute, seconde) — pas de paramètres optionnels en Kastel. Ces
// fonctions couvrent les deux cas d'usage qui n'ont pas de
// composants à fournir.
// ------------------------------------------------------------------

export func now() {
    return from_timestamp(clock());
}

export func from_timestamp(seconds) {
    let dt = new DateTime(1970, 1, 1, 0, 0, 0);
    dt.value = seconds;

    return dt;
}

// Parse le format produit par to_string() : "YYYY-MM-DD HH:MM:SS".
export func from_iso(text) {
    let parts = text.split(" ");
    let date_parts = parts.get(0).split("-");
    let time_parts = parts.get(1).split(":");

    return new DateTime(
        int(date_parts.get(0)),
        int(date_parts.get(1)),
        int(date_parts.get(2)),
        int(time_parts.get(0)),
        int(time_parts.get(1)),
        int(time_parts.get(2))
    );
}
