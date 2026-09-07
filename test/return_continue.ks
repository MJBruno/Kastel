println("TEST RETURN + CONTINUE");

function find_value() {
    let i = 0;

    while i < 10 {
        i += 1;

        if i < 5 {
            continue;
        }

        if i == 7 {
            return i;
        }
    }
 
    return -1;
}

println(find_value());