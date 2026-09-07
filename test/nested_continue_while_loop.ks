println("TEST CONTINUE WHILE IMBRIQUES");

let i = 0;

while i < 3 {
    let j = 0;

    while j < 5 {
        j += 1;

        if j == 2 {
            continue;
        }

        println(j);
    }

    i += 1;
}