let s = "café😀";

s.byte_at(4); // 169
s.byte_size(); // 9
s[0]; // "c"
s[3]; // "é"
s[4]; // "😀"

let copy = s.copy();
