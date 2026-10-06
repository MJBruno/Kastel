// std/io.ks
// E/S console officielle : les opérations restent des natives du langage.

export func print_value(value: dynamic) -> dynamic {
    return print(value);
}

export func println_value(value: dynamic) -> dynamic {
    return println(value);
}

export func read_line() -> str {
    return input();
}

export func read_line_with_prompt(prompt: str) -> str {
    return input(prompt);
}
