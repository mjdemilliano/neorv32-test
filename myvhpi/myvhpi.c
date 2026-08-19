#include <vhpi_ext_nvc.h>
#include <vhpi_user.h>

#include <stddef.h>

// NOTE: To see more debug information printed, run with environment variable NVC_VHPI_VERBOSE=1

static void gpio_out_changed(const struct vhpiCbDataS *value) {
    vhpi_printf("GPIO changed!");
    if (!value) {
        vhpi_assert(vhpiError, "value is null: %p", value);
        return;
    }
    vhpi_printf("value->obj = %p, value->user_data = %p, value->value = %p", value->obj, value->user_data, value->value);
    if (!value->obj) {
        vhpi_assert(vhpiError, "value->obj is null: %p", value->obj);
        return;
    }
    if (!value->user_data) {
        vhpi_assert(vhpiError, "value->user_data is null: %p", value->obj);
        return;
    }
    // if (!value->value) {
    //     vhpi_assert(vhpiError, "value->value is null: %p", value->value);
    //     return;
    // }
    // vhpi_printf(" => value = %d", value->value->value.intg);

    const char *kind_name = (const char*) vhpi_get_str(vhpiKindStrP, value->obj);
    vhpi_printf("kind name: %s", kind_name);
    int num_dimensions = vhpi_get(vhpiNumDimensionsP, value->obj);
    vhpi_printf("num dimensions: %d", num_dimensions);

    vhpiHandleT gpio_out_type = vhpi_handle(vhpiType, value->obj);
    vhpi_printf("type name: %s", vhpi_get_str(vhpiFullNameP, gpio_out_type));

    // vhpiHandleT gpio_out_constrs = vhpi_iterator(vhpiConstraints, gpio_out_type);
    // vhpiHandleT gpio_out_range = vhpi_scan(gpio_out_constrs);
    // vhpi_printf("left bound: %d, right bound: %d", vhpi_get(vhpiLeftBoundP, gpio_out_range), vhpi_get(vhpiRightBoundP, gpio_out_range));


    vhpiEnumT gpio_out_values[32] = {0};
    vhpiValueT gpio_out_value = {
        .format = vhpiLogicVecVal,
        .bufSize = 32 * sizeof(vhpiEnumT),
        .value = {
            .enumvs = gpio_out_values,
        }
    };
    int result = vhpi_get_value(value->obj, &gpio_out_value);
    if (0 != result) {
        vhpi_assert(vhpiError, "error getting gpio_out value (result=%d)");
        return;
    };
    const char *val_31 = "?";
    switch (gpio_out_value.value.enumvs[31]) {
        case vhpi0:
            val_31 = "0";
            break;
        case vhpi1:
            val_31 = "1";
            break;
        default:
            val_31 = "s";
    }
    vhpi_printf("gpio_out value: [0]=%d [31]=%d [31]=%s", gpio_out_value.value.enumvs[0], gpio_out_value.value.enumvs[31], val_31);

    unsigned char gpio_out_value_str_buf[1025] = {0};
    vhpiValueT gpio_out_value_str = {
        .format = vhpiStrVal,
        .bufSize = 1024,
        .value = {
            .str = gpio_out_value_str_buf,
        },
    };
    result = vhpi_get_value(value->obj, &gpio_out_value_str);
    if (0 != result) {
        vhpi_assert(vhpiError, "error getting gpio_out value (result=%d)");
        return;
    };
    vhpi_printf("gpio_out value: %s", (const char*) gpio_out_value_str.value.str);

    // if (!value->value) {
    //     vhpi_assert(vhpiError, "value->value is null: %p", value->value);
    //     return;
    // }
    // vhpi_printf(" => value = %d", value->value->value.intg);
}

static void end_of_init(const vhpiCbDataT *cb_data) {
    vhpi_printf("End of initialization");

    vhpiHandleT root = vhpi_handle(vhpiRootInst, NULL);

    vhpiHandleT gpio_out = vhpi_handle_by_name("gpio_out", root);
    if (!gpio_out) {
        vhpi_assert(vhpiError, "error getting gpio_out handle");
        return;
    }
    vhpi_printf("gpio_out obj handle: %p", gpio_out);
    // vhpiValueT gpio_out_value = {0};
    // if (0 != vhpi_get_value(gpio_out, &gpio_out_value)) {
    //     vhpi_assert(vhpiError, "error getting gpio_out value");
    //     return;
    // };
    // vhpi_printf("gpio_out value: %d", gpio_out_value.value.intg);

    vhpiCbDataT cb_gpio_out_change = {
        .reason = vhpiCbValueChange,
        .cb_rtn = gpio_out_changed,
        .obj = gpio_out,
        .time = (vhpiTimeT *)-1,
        .user_data = gpio_out,
    };
    if (!vhpi_register_cb(&cb_gpio_out_change, 0)) {
        vhpi_assert(vhpiError, "error registering callback for gpio_out value change!");
        return;
    }
    vhpi_printf("Callback registered");
}

static void shared_startup(void) {
    vhpi_printf("Hello world from VHPI plugin!");

    vhpiCbDataT cb_end_of_init = {
        .reason = vhpiCbEndOfInitialization,
        .cb_rtn = end_of_init,
    };
    vhpi_register_cb(&cb_end_of_init, vhpiReturnCb);
}

void (*vhpi_startup_routines[])() = {
   shared_startup,
   NULL
};
