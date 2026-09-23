// test/std/test_datetime.ks

import std.datetime;
from std.testing import assert_eq, assert_true, assert_false, run_tests;

// Epoch Unix : 1970-01-01 00:00:00 UTC.
func test_epoch_zero() {
    let dt = datetime.from_timestamp(0);

    assert_eq(dt.timestamp(), 0, "timestamp() renvoie la valeur passée au constructeur");
    assert_eq(dt.year(), 1970, "year() à l'epoch");
    assert_eq(dt.month(), 1, "month() à l'epoch");
    assert_eq(dt.day(), 1, "day() à l'epoch");
    assert_eq(dt.hour(), 0, "hour() à l'epoch");
    assert_eq(dt.minute(), 0, "minute() à l'epoch");
    assert_eq(dt.second(), 0, "second() à l'epoch");
}

// 1 000 000 000 -> 2001-09-09 01:46:40 UTC (le "milliard de secondes",
// une date de référence bien connue).
func test_known_timestamp() {
    let dt = datetime.from_timestamp(1000000000);

    assert_eq(dt.year(), 2001, "year() pour 1e9");
    assert_eq(dt.month(), 9, "month() pour 1e9");
    assert_eq(dt.day(), 9, "day() pour 1e9");
    assert_eq(dt.hour(), 1, "hour() pour 1e9");
    assert_eq(dt.minute(), 46, "minute() pour 1e9");
    assert_eq(dt.second(), 40, "second() pour 1e9");
}

// Régression : `initialize()` créait autrefois un champ `date` qui masquait
// la méthode `date()` du même nom -> tout appel à .date()/.toString()
// après construction plantait. Ce test échoue si la régression revient.
func test_date_and_time_and_tostring_do_not_crash() {
    let dt = datetime.from_timestamp(0);

    assert_eq(dt.date(), "1970-1-1", "date() après construction");
    assert_eq(dt.time(), "0:0:0", "time() après construction");
    assert_eq(dt.toString(), "1970-1-1 0:0:0", "toString() combine date() et time()");

    // Deuxième appel : si un champ masquant avait été recréé quelque part,
    // ce serait ici qu'il casserait.
    assert_eq(dt.date(), "1970-1-1", "date() reste appelable une seconde fois");
}

func test_leap_year() {
    let dt = datetime.from_timestamp(0);

    assert_true(dt.isLeapYear(2000), "2000 est bissextile (divisible par 400)");
    assert_false(dt.isLeapYear(1900), "1900 n'est pas bissextile (divisible par 100, pas 400)");
    assert_true(dt.isLeapYear(2024), "2024 est bissextile (divisible par 4)");
    assert_false(dt.isLeapYear(2023), "2023 n'est pas bissextile");

    assert_eq(dt.daysInYear(2000), 366, "daysInYear: année bissextile");
    assert_eq(dt.daysInYear(2001), 365, "daysInYear: année normale");
}

func test_days_in_month() {
    let dt = datetime.from_timestamp(0);

    assert_eq(dt.daysInMonth(2000, 2), 29, "février d'une année bissextile");
    assert_eq(dt.daysInMonth(2001, 2), 28, "février d'une année normale");
    assert_eq(dt.daysInMonth(2000, 1), 31, "janvier");
    assert_eq(dt.daysInMonth(2000, 4), 30, "avril");
    assert_eq(dt.daysInMonth(2000, 12), 31, "décembre");
}

// Le constructeur surchargé initialize(value) doit produire exactement le
// même résultat que new DateTime() + addSeconds(value) à partir de l'epoch.
func test_arithmetic_returns_new_instance() {
    let base = datetime.from_timestamp(0);

    let plus_seconds = base.addSeconds(30);
    assert_eq(plus_seconds.timestamp(), 30, "addSeconds");
    assert_eq(base.timestamp(), 0, "addSeconds ne modifie pas l'instance d'origine");

    let plus_minutes = base.addMinutes(1);
    assert_eq(plus_minutes.timestamp(), 60, "addMinutes");

    let plus_hours = base.addHours(1);
    assert_eq(plus_hours.timestamp(), 3600, "addHours");

    let plus_days = base.addDays(1);
    assert_eq(plus_days.timestamp(), 86400, "addDays");
    assert_eq(plus_days.year(), 1970, "addDays: year inchangé");
    assert_eq(plus_days.month(), 1, "addDays: month inchangé");
    assert_eq(plus_days.day(), 2, "addDays: day incrémenté");
}

func test_addDays_crosses_month_boundary() {
    // 1970-01-31 + 1 jour -> 1970-02-01.
    let jan_31 = datetime.from_timestamp(30 * 86400); // 30 jours écoulés depuis le 1er janvier -> le 31
    assert_eq(jan_31.month(), 1, "sanity: on part bien de janvier");
    assert_eq(jan_31.day(), 31, "sanity: le 31");

    let next = jan_31.addDays(1);
    assert_eq(next.month(), 2, "addDays franchit la limite du mois");
    assert_eq(next.day(), 1, "addDays: repart à 1");
}

func test_now_is_a_plausible_current_timestamp() {
    let dt = datetime.now();
    // Le 2020-01-01 (1577836800) comme plancher très permissif : suffit à
    // vérifier que now() n'est pas resté bloqué à 0 ou une valeur absurde,
    // sans dépendre d'une date précise au moment du test.
    assert_true(dt.timestamp() > 1577836800, "now() renvoie un timestamp plausible");
}

run_tests([
    ["epoch zero", test_epoch_zero],
    ["known timestamp (1e9)", test_known_timestamp],
    ["date()/time()/toString() ne plantent pas (régression)", test_date_and_time_and_tostring_do_not_crash],
    ["leap year", test_leap_year],
    ["days in month", test_days_in_month],
    ["arithmetic returns a new instance", test_arithmetic_returns_new_instance],
    ["addDays crosses month boundary", test_addDays_crosses_month_boundary],
    ["now() is plausible", test_now_is_a_plausible_current_timestamp],
]);
