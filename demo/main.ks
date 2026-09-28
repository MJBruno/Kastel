let mutex = mutex();

func worker() {
    mutex.lock();

    // section critique

    mutex.unlock();
}