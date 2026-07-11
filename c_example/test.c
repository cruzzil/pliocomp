#include <assert.h>
#include "pliocomp.c"
#include "pliocomp_no_goto.c"

void test_1() {
    int input[] = {3,56,3343,22225,3,66,3,3,3};
    int xs = 1;
    short compressed[200];
    for(int i=0; i < 200; i++) {
        compressed[i] = 0;
    }

    int npix = 9;
    int res = pl_p2li(&input, xs, &compressed, npix);

    printf("Compressed items: %d\n", res);

    int uncompressed[10];
    for(int i=0; i < 10; i++) {
        uncompressed[i] = 0;
    }

    int res2 = pl_l2pi(&compressed, xs, &uncompressed, npix);

    printf("Uncompressed items: %d\n", res2);
}

void test_2() {
    int input[] = {3,56,3343,22225,3,66,3,3,3};
    int xs = 0;
    short compressed[200];
    for(int i=0; i < 200; i++) {
        compressed[i] = 0;
    }

    int npix = 9;
    int res = pl_p2li_new(&input, xs, &compressed, npix);

    printf("Compressed items: %d\n", res);

    int uncompressed[10];
    for(int i=0; i < 10; i++) {
        uncompressed[i] = 0;
    }

    int res2 = pl_l2pi_new(&compressed, xs, &uncompressed, npix);

    printf("Uncompressed items: %d\n", res2);
}

int main() {

    test_1();

    return 0;
}