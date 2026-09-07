// ============================================================
// TEST 1 — IF / ELSE
// ============================================================

println("TEST IF / ELSE");

if true {
    println("if: OK");
} else {
    println("if: ERROR");
}

if false {
    println("else: ERROR");
} else {
    println("else: OK");
}


// ============================================================
// TEST 2 — IF / ELSE IMBRIQUÉ
// ============================================================

println("TEST IF IMBRIQUÉ");

let x = 10;

if x > 5 {
    if x == 10 {
        println("nested if: OK");
    } else {
        println("nested if: ERROR");
    }
} else {
    println("nested if: ERROR");
}


// ============================================================
// TEST 3 — WHILE
// ============================================================

println("TEST WHILE");

let i = 0;

while i < 5 {
    println(i);
    i += 1;
}


// ============================================================
// TEST 4 — WHILE + IF
// ============================================================

println("TEST WHILE + IF");

let j = 0;

while j < 5 {
    if j == 3 {
        println("found 3");
    }

    j += 1;
}


// ============================================================
// TEST 5 — FOR IN + ARRAY
// ============================================================

println("TEST FOR IN");

for value in [10, 20, 30, 40] {
    println(value);
}


// ============================================================
// TEST 6 — FOR IN + IF
// ============================================================

println("TEST FOR IN + IF");

for value in [1, 2, 3, 4, 5] {
    if value > 3 {
        println(value);
    }
}


// ============================================================
// TEST 7 — BREAK
// ============================================================

println("TEST BREAK");

let b = 0;

while b < 10 {
    if b == 5 {
        break;
    }

    println(b);
    b += 1;
}


// ============================================================
// TEST 8 — CONTINUE
// ============================================================

println("TEST CONTINUE");

let c = 0;

while c < 5 {
    c += 1;

    if c == 3 {
        continue;
    }

    println(c);
}


// ============================================================
// TEST 9 — BREAK DANS FOR IN
// ============================================================

println("TEST BREAK FOR");

for value in [1, 2, 3, 4, 5] {
    if value == 4 {
        break;
    }

    println(value);
}


// ============================================================
// TEST 10 — CONTINUE DANS FOR IN
// ============================================================

println("TEST CONTINUE FOR");

for value in [1, 2, 3, 4, 5] {
    if value == 3 {
        continue;
    }

    println(value);
}