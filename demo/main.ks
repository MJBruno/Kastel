// examples/std_extras_demo.ks
//
// std.json / std.file / std.path / std.os
// Lancer avec : kastel examples/std_extras_demo.ks

import std.json;
import std.file;
import std.path;
import std.os;

// -- json ---------------------------------------------------------------
let payload = {
    name: "Ada",
    age: 36,
    languages: ["Kastel", "Rust"],
    active: true
};

let text = json.encode(payload);
println(text);

// let decoded = json.decode(text);
// println(decoded.get("name"));
// println(decoded.get("languages"));

// // -- file + json.write_file/read_file ------------------------------------
let out = path.join([os.cwd(), "kastel_demo.json"]);

json.write_file(out, payload);
// println(file.read(out));

// let reloaded = json.read_file(out);
// println(reloaded.get("age"));

// file.delete(out);

// // -- path -----------------------------------------------------------------
// println(path.basename(out));     // kastel_demo.json
// println(path.extension(out));    // json
// println(path.stem(out));         // kastel_demo
// println(path.dirname(out));

// // -- os -------------------------------------------------------------------
// println(os.name());
// println(os.arch());
// println(os.args());
