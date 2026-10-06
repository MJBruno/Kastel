import std.datetime;
import std.regex;
import std.statistics;
import std.testing;
import std.string;

let date = datetime.DateTime.from_ymdhms_millis(2026, 10, 6, 23, 59, 59, 999);
match date {
    Ok(value) => {
        testing.assert_true(value.millisecond() == 999, "milliseconde conservee");
        testing.assert_true(value.add_milliseconds(1).millisecond() == 0, "passage de jour");
    }
    Err(error) => {
        throw error;
    }
}

match regex.is_match("^[A-Z][a-z]+$", "Kastel") {
    Ok(value) => {
        testing.assert_true(value, "regex match");
    }
    Err(error) => {
        throw error;
    }
}

match regex.find("(Kastel) ([0-9]+)", "Kastel 2026") {
    Ok(Some(value)) => {
        testing.assert_true(value.start == 0, "regex start");
        testing.assert_true(value.end == 11, "regex end exclusif");
        testing.assert_true(value.groups.size() == 2, "regex groups");
    }
    _ => {
        throw "regex find failed";
    }
}

match statistics.linear_regression([1.0, 2.0, 3.0], [2.0, 4.0, 6.0]) {
    Ok(value) => {
        testing.assert_true(value.slope == 2.0, "regression slope");
        testing.assert_true(value.intercept == 0.0, "regression intercept");
    }
    Err(error) => {
        throw error;
    }
}

match string.parse_int("42") {
    Ok(value) => {
        testing.assert_true(value == 42, "string parse int");
    }
    Err(error) => {
        throw error;
    }
}

println("std 1.0 smoke tests: ok");
