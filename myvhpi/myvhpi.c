#include <vhpi_ext_nvc.h>
#include <vhpi_user.h>

#include <stddef.h>

static void shared_startup(void) {
    vhpi_printf("Hello world from VHPI plugin!");
}

void (*vhpi_startup_routines[])() = {
   shared_startup,
   NULL
};
