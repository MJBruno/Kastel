// std/datetime.ks
//
// Date et heure civiles UTC, avec une précision de stockage et de
// construction à la MILLISECONDE.
//
// Le module s'appuie uniquement sur clock() (secondes Unix en float) et
// l'arithmétique entière. La classe DateTime stocke toujours le nombre
// de millisecondes depuis 1970-01-01T00:00:00Z.
//
// Précision contractuelle :
//   - DateTime conserve exactement les millisecondes fournies.
//   - now_millis() convertit la fraction de seconde fournie par clock().
//   - aucune précision microseconde/nanoseconde n'est prétendue.
//
// La conversion jours <-> date civile (année/mois/jour) utilise
// l'algorithme de Howard Hinnant (domaine public), correct sur le
// calendrier grégorien proleptique, y compris avant 1970 et pour les
// années négatives.

// ------------------------------------------------------------------
// Arithmétique entière auxiliaire
// ------------------------------------------------------------------

// Division entière arrondie vers -infini (idiv natif).
func floor_div(a: int, b: int) -> int {
    return idiv(a, b);
}

// Reste normalisé dans [0, b) pour b > 0.
func floor_mod(a: int, b: int) -> int {
    return a - idiv(a, b) * b;
}

// ------------------------------------------------------------------
// Calendrier et validation
// ------------------------------------------------------------------

export func is_gregorian_leap_year(year: int) -> bool {
    if floor_mod(year, 4) != 0 {
        return false;
    }
    if floor_mod(year, 100) != 0 {
        return true;
    }
    return floor_mod(year, 400) == 0;
}

// Nombre de jours dans le mois.
// Retourne 0 pour un mois hors de 1..12.
export func days_in_month(year: int, month: int) -> int {
    if month < 1 || month > 12 {
        return 0;
    }
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

export func is_valid_date(year: int, month: int, day: int) -> bool {
    if month < 1 || month > 12 {
        return false;
    }
    let max_day = days_in_month(year, month);
    return day >= 1 && day <= max_day;
}

export func validate_date(year: int, month: int, day: int) -> Result<bool, str> {
    if month < 1 || month > 12 {
        return Err("mois invalide: " + str(month));
    }

    let max_day = days_in_month(year, month);
    if day < 1 || day > max_day {
        return Err(
            "jour invalide pour " + str(year) + "-" + pad2(month) + ": " + str(day)
        );
    }

    return Ok(true);
}

export func is_valid_time(
    hour: int,
    minute: int,
    second: int,
    millisecond: int
) -> bool {
    return hour >= 0 && hour <= 23
        && minute >= 0 && minute <= 59
        && second >= 0 && second <= 59
        && millisecond >= 0 && millisecond <= 999;
}

export func validate_time(
    hour: int,
    minute: int,
    second: int,
    millisecond: int
) -> Result<bool, str> {
    if hour < 0 || hour > 23 {
        return Err("heure invalide: " + str(hour));
    }
    if minute < 0 || minute > 59 {
        return Err("minute invalide: " + str(minute));
    }
    if second < 0 || second > 59 {
        return Err("seconde invalide: " + str(second));
    }
    if millisecond < 0 || millisecond > 999 {
        return Err("milliseconde invalide: " + str(millisecond));
    }
    return Ok(true);
}

// ------------------------------------------------------------------
// Conversion date civile <-> jours
// ------------------------------------------------------------------

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

// (année, mois, jour) -> jours écoulés depuis 1970-01-01.
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

// ------------------------------------------------------------------
// Formatage et horodatage courant
// ------------------------------------------------------------------

func pad2(n: int) -> str {
    if n < 10 {
        return "0" + str(n);
    }
    return str(n);
}

func pad3(n: int) -> str {
    if n < 10 {
        return "00" + str(n);
    }
    if n < 100 {
        return "0" + str(n);
    }
    return str(n);
}

// Horodatage courant en millisecondes depuis Unix epoch UTC.
// Pour mesurer une durée, préférer la différence de deux DateTime/temps
// plutôt qu'une horloge monotone : clock() est une horloge civile.
export func now_millis() -> int {
    return floor(clock() * 1000.0);
}

// ------------------------------------------------------------------
// DateTime
// ------------------------------------------------------------------

export class DateTime {
    private let epoch_millis: int = 0;

    private func initialize(epoch_millis: int) {
        self.epoch_millis = epoch_millis;
    }

    static func now() -> DateTime {
        return new DateTime(now_millis());
    }

    static func from_timestamp(epoch_seconds: int) -> DateTime {
        return new DateTime(epoch_seconds * 1000);
    }

    static func from_timestamp_millis(epoch_millis: int) -> DateTime {
        return new DateTime(epoch_millis);
    }

    static func from_ymd(year: int, month: int, day: int) -> Result<DateTime, str> {
        return DateTime.from_ymdhms_millis(year, month, day, 0, 0, 0, 0);
    }

    static func from_ymdhms(
        year: int,
        month: int,
        day: int,
        hour: int,
        minute: int,
        second: int
    ) -> Result<DateTime, str> {
        return DateTime.from_ymdhms_millis(year, month, day, hour, minute, second, 0);
    }

    static func from_ymdhms_millis(
        year: int,
        month: int,
        day: int,
        hour: int,
        minute: int,
        second: int,
        millisecond: int
    ) -> Result<DateTime, str> {
        match validate_date(year, month, day) {
            Ok(_) => {
                // Validation complète de l'heure ci-dessous.
            }
            Err(error) => {
                return Err(error);
            }
        }

        match validate_time(hour, minute, second, millisecond) {
            Ok(_) => {
                // Tous les composants temporels sont valides.
            }
            Err(error) => {
                return Err(error);
            }
        }

        let days = days_from_civil(year, month, day);
        let millis = days * 86400000
            + hour * 3600000
            + minute * 60000
            + second * 1000
            + millisecond;

        return Ok(new DateTime(millis));
    }

    // Secondes depuis epoch, arrondies vers -infini.
    func timestamp() -> int {
        return floor_div(self.epoch_millis, 1000);
    }

    // Millisecondes exactes depuis epoch.
    func timestamp_millis() -> int {
        return self.epoch_millis;
    }

    func days_since_epoch() -> int {
        return floor_div(self.epoch_millis, 86400000);
    }

    func millis_of_day() -> int {
        return self.epoch_millis - self.days_since_epoch() * 86400000;
    }

    func year() -> int {
        match civil_from_days(self.days_since_epoch()) {
            (y, _, _) => {
                return y;
            }
        }
    }

    func month() -> int {
        match civil_from_days(self.days_since_epoch()) {
            (_, m, _) => {
                return m;
            }
        }
    }

    func day() -> int {
        match civil_from_days(self.days_since_epoch()) {
            (_, _, d) => {
                return d;
            }
        }
    }

    // Composantes horaires garanties dans les plages :
    // hour 0..23, minute 0..59, second 0..59, millisecond 0..999.
    func hour() -> int {
        return idiv(self.millis_of_day(), 3600000);
    }

    func minute() -> int {
        return idiv(self.millis_of_day() % 3600000, 60000);
    }

    func second() -> int {
        return idiv(self.millis_of_day() % 60000, 1000);
    }

    func millisecond() -> int {
        return self.millis_of_day() % 1000;
    }

    // Millisecondes écoulées depuis minuit.
    func time_millis() -> int {
        return self.millis_of_day();
    }

    // 0 = dimanche ... 6 = samedi (1970-01-01 était un jeudi).
    func weekday() -> int {
        return floor_mod(self.days_since_epoch() + 4, 7);
    }

    func is_weekend() -> bool {
        let weekday = self.weekday();
        return weekday == 0 || weekday == 6;
    }

    func is_leap_year() -> bool {
        return is_gregorian_leap_year(self.year());
    }

    func add_milliseconds(delta: int) -> DateTime {
        return new DateTime(self.epoch_millis + delta);
    }

    func add_seconds(delta: int) -> DateTime {
        return self.add_milliseconds(delta * 1000);
    }

    func add_minutes(delta: int) -> DateTime {
        return self.add_milliseconds(delta * 60000);
    }

    func add_hours(delta: int) -> DateTime {
        return self.add_milliseconds(delta * 3600000);
    }

    func add_days(delta: int) -> DateTime {
        return self.add_milliseconds(delta * 86400000);
    }

    // Différences signées : self - other.
    func difference_millis(other: DateTime) -> int {
        return self.epoch_millis - other.epoch_millis;
    }

    func difference_seconds(other: DateTime) -> float {
        return self.difference_millis(other) / 1000.0;
    }

    func difference_minutes(other: DateTime) -> float {
        return self.difference_millis(other) / 60000.0;
    }

    func difference_hours(other: DateTime) -> float {
        return self.difference_millis(other) / 3600000.0;
    }

    func to_string() -> str {
        return str(self.year()) + "-" + pad2(self.month()) + "-" + pad2(self.day())
            + " " + pad2(self.hour()) + ":" + pad2(self.minute()) + ":" + pad2(self.second());
    }

    // Format proche ISO 8601, en UTC implicite, avec millisecondes.
    func to_iso_string() -> str {
        return str(self.year()) + "-" + pad2(self.month()) + "-" + pad2(self.day())
            + "T" + pad2(self.hour()) + ":" + pad2(self.minute()) + ":" + pad2(self.second())
            + "." + pad3(self.millisecond());
    }

    func equals(other: DateTime) -> bool {
        return self.epoch_millis == other.epoch_millis;
    }

    func compare(other: DateTime) -> int {
        if self.epoch_millis < other.epoch_millis {
            return -1;
        }
        if self.epoch_millis > other.epoch_millis {
            return 1;
        }
        return 0;
    }
}
