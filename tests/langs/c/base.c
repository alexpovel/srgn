#include <stdio.h>
#include "base.h"

typedef unsigned int uint;
typedef void (*callback)(void);

/* Multiline comment.
 * New line.
 */
struct S {
    int a;
    int b;
    const char *c;
    callback cb;
};

union U {
    char test[4];
    int a;
};

/**
 * Doxygen comment
 *
 */
enum E {
    A, ///< Doxygen comment.
    B, /*< Doxygen comment. */
    C,
};


extern int external_var;

const char *external_function_declaration(const void *ptr);

// Main function.
int main(void) {
    int a = 0; /* C Stype comments */
    struct S s;
    struct S *sp;
    union U u;

    // Call a function.
    printf("Hello, World!\n");
    s.cb();
    sp->cb();

    if (a) {
        printf("a\n");
    } else if (sp) {
        printf("b\n");
    } else {
        printf("c\n");
    }

    for (int a = 0; a < 10; a++) {
        printf("for\n");
    }

    while (a++ < 100) {
        printf("while\n");
    }

    do {
        printf("do-while\n");
    } while (0);

    switch (a) {
        case 1:
            return 0;
        default:
            return -1;
    }

    return 0;
}
