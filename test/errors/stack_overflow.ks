// À placer dans test/errors/stack_overflow.ks
// Doit échouer PROPREMENT (RuntimeError::StackOverflow), sans que le
// processus natif ne crashe (segfault) ni ne boucle indéfiniment.

func loop(n: int) -> int {
    return loop(n + 1);
}

print(loop(0));
