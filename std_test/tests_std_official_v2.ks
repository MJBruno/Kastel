import std.string;
import std.statistics;
import std.datetime;
import std.fs;
import std.net;
import std.http;

let title = string.capitalize("kastel");
let left = string.pad_left("42", 5, "0");
let right = string.pad_right("ks", 5, ".");
let mode = statistics.mode([1, 2, 2, 3]).unwrap();
let dt = DateTime.from_ymdhms_millis(2026, 10, 6, 15, 13, 59, 999).unwrap();
let next = dt.add_milliseconds(1);

if title != "Kastel" {
    throw "std.string.capitalize failed";
}
if left.unwrap() != "00042" {
    throw "std.string.pad_left failed";
}
if right.unwrap() != "ks..." {
    throw "std.string.pad_right failed";
}
if mode != 2 {
    throw "std.statistics.mode failed";
}
if next.millisecond() != 0 || next.second() != 0 {
    throw "std.datetime millisecond rollover failed";
}

println("Kastel official std smoke tests: ok");
