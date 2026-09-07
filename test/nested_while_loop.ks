println("TEST WHILE IMBRIQUES");

let i = 0;

while i < 3 {
    let j = 0;

    while j < 5 {
        if j == 2 {
            break;
        }

        println(j);
        j += 1;
    }

    i += 1;
}