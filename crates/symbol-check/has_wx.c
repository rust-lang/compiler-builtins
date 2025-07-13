void intermediate(void (*)(int, int), int);

int hack(int *array, int size) {
    void store (int index, int value) {
        array[index] = value;
    }

    intermediate(store, size);
}
