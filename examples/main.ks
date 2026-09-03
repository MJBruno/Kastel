let flags = 0b1010;
let mask  = 0xFF;

println(flags & mask);   // ET bitwise
println(flags | mask);   // OU bitwise
println(flags ^ mask);   // XOR bitwise
println(~flags);         // NON bitwise
println(1 << 4);         // 16
println(256 >> 4);       // 16

let x = 10;
x += 5;   // 15
x -= 3;   // 12
x *= 2;   // 24
x /= 4;   // 6
x %= 4;   // 2