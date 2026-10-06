// Smoke tests for std.datetime.
from std.datetime import DateTime, is_valid_time, is_valid_date;
from std.testing import assert_eq, assert_true, assert_false;

let value_result = DateTime.from_ymdhms_millis(2026, 10, 6, 15, 42, 17, 235);
match value_result {
    Ok(value) => {
        assert_eq(value.year(), 2026, "datetime smoke test");
        assert_eq(value.month(), 10, "datetime smoke test");
        assert_eq(value.day(), 6, "datetime smoke test");
        assert_eq(value.hour(), 15, "datetime smoke test");
        assert_eq(value.minute(), 42, "datetime smoke test");
        assert_eq(value.second(), 17, "datetime smoke test");
        assert_eq(value.millisecond(), 235, "datetime smoke test");
        assert_eq(value.time_millis(), 56537235, "time_millis doit conserver les millisecondes depuis minuit");
        assert_eq(value.add_milliseconds(765).millisecond(), 0, "datetime smoke test");
        assert_eq(value.add_minutes(1).minute(), 43, "datetime smoke test");
        assert_eq(value.add_hours(1).hour(), 16, "datetime smoke test");
    }
    Err(error) => {
        throw error;
    }
}

assert_true(is_valid_time(23, 59, 59, 999), "23:59:59.999 doit être valide");
assert_false(is_valid_time(24, 0, 0, 0), "24:00:00.000 doit être invalide");
assert_false(is_valid_time(23, 60, 0, 0), "minute 60 doit être invalide");
assert_false(is_valid_time(23, 59, 60, 0), "seconde 60 doit être invalide");
assert_false(is_valid_time(23, 59, 59, 1000), "milliseconde 1000 doit être invalide");

assert_true(is_valid_date(2024, 2, 29), "2024-02-29 doit être valide");
assert_false(is_valid_date(2023, 2, 29), "2023-02-29 doit être invalide");
assert_false(is_valid_date(2026, 13, 1), "mois 13 doit être invalide");

match DateTime.from_ymdhms_millis(2026, 10, 6, 23, 59, 59, 999) {
    Ok(value) => {
        let next = value.add_milliseconds(1);
        assert_eq(next.year(), 2026, "datetime smoke test");
        assert_eq(next.month(), 10, "datetime smoke test");
        assert_eq(next.day(), 7, "datetime smoke test");
        assert_eq(next.hour(), 0, "datetime smoke test");
        assert_eq(next.minute(), 0, "datetime smoke test");
        assert_eq(next.second(), 0, "datetime smoke test");
        assert_eq(next.millisecond(), 0, "datetime smoke test");
    }
    Err(error) => {
        throw error;
    }
}

println("std.datetime smoke tests: ok");
