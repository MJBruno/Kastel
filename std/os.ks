// std/os.ks
// Interaction officielle avec l'environnement du processus.

export func platform_name() -> str {
    return os_name();
}

export func architecture() -> str {
    return os_arch();
}

export func arguments() -> List<str> {
    return args();
}

export func terminate(code: int) -> None {
    exit(code);
}

export func environment(name: str) -> dynamic {
    return env(name);
}

export func current_directory() -> str {
    return cwd();
}

export func clock_seconds() -> float {
    return clock();
}
