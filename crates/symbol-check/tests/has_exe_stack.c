/* GNU nested functions are the only way I could find to force an explicitly
 * executable stack. Supported by GCC only, not Clang. */

void intermediate(void (*)(int, int), int);

int hack(int *array, int size) {
    void store (int index, int value) {
        array[index] = value;
    }

    intermediate(store, size);
}
