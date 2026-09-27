// std/datetime.ks
//
// Kastel n'a pas de type date/heure natif : ce module en construit un
// au-dessus de deux briques natives seulement (clock(), qui renvoie
// les secondes écoulées depuis epoch Unix UTC, et l'arithmétique
// entière). La conversion jours <-> date civile (année/mois/jour)
// utilise l'algorithme de Howard Hinnant (domaine public,
// http://howardhinnant.github.io/date_algorithms.html), correct sur
// tout le calendrier grégorien proleptique, dates avant 1970 et années
// négatives incluses.

// ------------------------------------------------------------------
// Arithmétique entière auxiliaire
// ------------------------------------------------------------------

// Division entière arrondie vers -infini (idiv natif fait déjà ça,
// simple alias pour la lisibilité du calendrier ci-dessous).
func floor_div(a: int, b: int) -> int {
    return idiv(a, b);
}

// `%` seul peut renvoyer un reste négatif pour un dividende négatif
// (ex. années avant 1970) ; le calendrier a besoin d'un reste toujours
// positif.
func floor_mod(a: int, b: int) -> int {
    return a - idiv(a, b) * b;
}

export func is_gregorian_leap_year(year: int) -> bool {
    if floor_mod(year, 4) != 0 {
        return false;
    }
    if floor_mod(year, 100) != 0 {
        return true;
    }
    return floor_mod(year, 400) == 0;
}

export func days_in_month(year: int, month: int) -> int {
    if month == 1 || month == 3 || month == 5 || month == 7
        || month == 8 || month == 10 || month == 12 {
        return 31;
    }
    if month == 4 || month == 6 || month == 9 || month == 11 {
        return 30;
    }
    if is_gregorian_leap_year(year) {
        return 29;
    }
    return 28;
}

// Jours écoulés depuis 1970-01-01 -> (année, mois, jour).
func civil_from_days(days: int) {
    let z = days + 719468;
    let era = floor_div(z, 146097);
    let doe = z - era * 146097;
    let yoe = floor_div(
        doe - floor_div(doe, 1460) + floor_div(doe, 36524) - floor_div(doe, 146096),
        365
    );
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + floor_div(yoe, 4) - floor_div(yoe, 100));
    let mp = floor_div(5 * doy + 2, 153);
    let d = doy - floor_div(153 * mp + 2, 5) + 1;

    let m = mp - 9;
    if mp < 10 {
        m = mp + 3;
    }
    if m <= 2 {
        y = y + 1;
    }

    return (y, m, d);
}

// (année, mois, jour) -> jours écoulés depuis 1970-01-01. Réciproque
// exacte de civil_from_days pour toute date valide.
func days_from_civil(year: int, month: int, day: int) -> int {
    let y = year;
    if month <= 2 {
        y = y - 1;
    }

    let era = floor_div(y - 399, 400);
    if y >= 0 {
        era = floor_div(y, 400);
    }

    let yoe = y - era * 400;

    let mp = month + 9;
    if month > 2 {
        mp = month - 3;
    }

    let doy = floor_div(153 * mp + 2, 5) + day - 1;
    let doe = yoe * 365 + floor_div(yoe, 4) - floor_div(yoe, 100) + doy;

    return era * 146097 + doe - 719468;
}

func pad2(n: int) -> str {
    if n < 10 {
        return "0" + str(n);
    }
    return str(n);
}

// ------------------------------------------------------------------
// DateTime
// ------------------------------------------------------------------

export class DateTime {
    private let epoch_seconds: int = 0;

    // Constructeur privé : toute instance passe par une des fabriques
    // statiques ci-dessous, qui valident leurs entrées (from_ymdhms
    // renvoie un Result plutôt que de construire une date invalide).
    private func initialize(epoch_seconds: int) {
        self.epoch_seconds = epoch_seconds;
    }

    static func now() -> DateTime {
        return new DateTime(floor(clock()));
    }

    static func from_timestamp(epoch_seconds: int) -> DateTime {
        return new DateTime(epoch_seconds);
    }

    static func from_ymd(year: int, month: int, day: int) -> Result<DateTime, str> {
        return DateTime.from_ymdhms(year, month, day, 0, 0, 0);
    }

    static func from_ymdhms(
        year: int,
        month: int,
        day: int,
        hour: int,
        minute: int,
        second: int
    ) -> Result<DateTime, str> {
        if month < 1 || month > 12 {
            return Err("mois invalide: " + str(month));
        }

        let max_day = days_in_month(year, month);
        if day < 1 || day > max_day {
            return Err(
                "jour invalide pour " + str(year) + "-" + pad2(month) + ": " + str(day)
            );
        }
        if hour < 0 || hour > 23 {
            return Err("heure invalide: " + str(hour));
        }
        if minute < 0 || minute > 59 {
            return Err("minute invalide: " + str(minute));
        }
        if second < 0 || second > 59 {
            return Err("seconde invalide: " + str(second));
        }

        let days = days_from_civil(year, month, day);
        let seconds = days * 86400 + hour * 3600 + minute * 60 + second;

        return Ok(new DateTime(seconds));
    }

    func timestamp() -> int {
        return self.epoch_seconds;
    }

    func days_since_epoch() -> int {
        return floor_div(self.epoch_seconds, 86400);
    }

    func seconds_of_day() -> int {
        return self.epoch_seconds - self.days_since_epoch() * 86400;
    }

    func year() -> int {
        match civil_from_days(self.days_since_epoch()) {
            (y, m, d) => {
                return y;
            }
        }
    }

    func month() -> int {
        match civil_from_days(self.days_since_epoch()) {
            (y, m, d) => {
                return m;
            }
        }
    }

    func day() -> int {
        match civil_from_days(self.days_since_epoch()) {
            (y, m, d) => {
                return d;
            }
        }
    }

    func hour() -> int {
        return idiv(self.seconds_of_day(), 3600);
    }

    func minute() -> int {
        return idiv(self.seconds_of_day() % 3600, 60);
    }

    func second() -> int {
        return self.seconds_of_day() % 60;
    }

    // 0 = dimanche ... 6 = samedi (1970-01-01 était un jeudi, d'où +4).
    func weekday() -> int {
        return floor_mod(self.days_since_epoch() + 4, 7);
    }

    func is_weekend() -> bool {
        let day = self.weekday();
        return day == 0 || day == 6;
    }

    func is_leap_year() -> bool {
        return is_gregorian_leap_year(self.year());
    }

    func add_seconds(delta: int) -> DateTime {
        return new DateTime(self.epoch_seconds + delta);
    }

    func add_days(delta: int) -> DateTime {
        return self.add_seconds(delta * 86400);
    }

    // Différence signée en secondes (self - other), positive si self
    // est postérieur à other.
    func difference_seconds(other: DateTime) -> int {
        return self.epoch_seconds - other.epoch_seconds;
    }

    func to_string() -> str {
        return str(self.year()) + "-" + pad2(self.month()) + "-" + pad2(self.day())
            + " " + pad2(self.hour()) + ":" + pad2(self.minute()) + ":" + pad2(self.second());
    }

    // Nommées `equals`/`compare` : Kastel branche automatiquement
    // ==, <, <=, >, >= dessus (capabilities Eq/Ord), pas besoin de les
    // déclarer explicitement ailleurs.
    func equals(other: DateTime) -> bool {
        return self.epoch_seconds == other.epoch_seconds;
    }

    func compare(other: DateTime) -> int {
        if self.epoch_seconds < other.epoch_seconds {
            return -1;
        }
        if self.epoch_seconds > other.epoch_seconds {
            return 1;
        }
        return 0;
    }
}
