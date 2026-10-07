import std.datetime;

println(datetime.is_gregorian_leap_year(2024));
println(datetime.days_in_month(2024, 2));
println(datetime.is_valid_date(2026, 10, 7));
println(datetime.is_valid_time(12, 30, 15, 250));

match datetime.DateTime.from_ymdhms_millis(2026, 10, 7, 12, 30, 15, 250) {
    Ok(date) => {
        println(date.year());
        println(date.month());
        println(date.day());
        println(date.hour());
        println(date.minute());
        println(date.second());
        println(date.millisecond());
        println(date.timestamp_millis());
        println(date.to_iso_string());
        println(date.add_days(1).to_iso_string());
    }
    Err(error) => println(error);
}

let now = datetime.DateTime.now();
println(now.year());
println(datetime.now_millis());

println("std.datetime: OK");
