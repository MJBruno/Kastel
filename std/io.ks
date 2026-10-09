// std/io.ks
// Console I/O et handles persistants de fichiers.

// Origine de positionnement pour File.seek(offset, origin).
export enum SeekFrom { Start, Current, End }

export func print_value(value: dynamic) -> str {
    return print(value);
}

export func println_value(value: dynamic) -> str {
    return println(value);
}

export func read_line() -> str {
    return input();
}

export func read_line_with_prompt(prompt: str) -> str {
    return input(prompt);
}

// Ouvre un fichier en lecture seule.
export func open_file(path: str) -> File {
    return io_file_open(path);
}

// Crée un fichier en écriture ; un fichier existant est tronqué.
export func create_file(path: str) -> File {
    return io_file_create(path);
}

// Constructeur du builder de configuration équivalent à OpenOptions.
export func open_options() -> OpenOptions {
    return io_open_options();
}
