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


// Lecteur tamponné : lit le fichier par blocs. read_line() retourne
// Option<str> ; None signifie que la fin du fichier est atteinte. buffer()
// renvoie les octets actuellement préchargés, sans les consommer.
export func buf_reader(path: str) -> BufReader {
    return io_buf_reader(path);
}

// Variante avec une capacité de buffer explicite (1 à 16 Mio).
export func buf_reader_with_capacity(path: str, capacity: int) -> BufReader {
    return io_buf_reader_with_capacity(path, capacity);
}

// Écrivain tamponné : crée le fichier ou tronque son contenu existant.
// Utiliser flush(), sync_all() ou close() pour traiter les erreurs d'écriture.
export func buf_writer(path: str) -> BufWriter {
    return io_buf_writer(path);
}

export func buf_writer_with_capacity(path: str, capacity: int) -> BufWriter {
    return io_buf_writer_with_capacity(path, capacity);
}

// Écrivain tamponné en ajout : conserve le contenu préexistant du fichier.
export func buf_writer_append(path: str) -> BufWriter {
    return io_buf_writer_append(path);
}

export func buf_writer_append_with_capacity(path: str, capacity: int) -> BufWriter {
    return io_buf_writer_append_with_capacity(path, capacity);
}
