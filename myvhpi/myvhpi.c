#include <vhpi_ext_nvc.h>
#include <vhpi_user.h>

#include <stddef.h>

// NOTE: To see more debug information printed, run with environment variable NVC_VHPI_VERBOSE=1

#define GPIO_OUT_SIZE 32  // We can also read this using the API but let's just hardcode the value.

/// Convert std_logic enum value to string representation.
static const char std_logic_value_to_str(vhpiEnumT value)
{
    // See `vhpi_user.h` for the definitions of vhpiU, vhpiX, etc.
    // This is also what `vhpi_to_string` does internally.
    static const char STD_LOGIC_VALUES[] = "UX01ZWLH-";
    if (value > vhpiDontCare) {
        return '?';
    } else {
        return STD_LOGIC_VALUES[value];
    }
}

static void gpio_out_changed(const struct vhpiCbDataS *value) {
    if (!value || !value->obj) {
        vhpi_assert(vhpiError, "value is null: value=%p, value->obj=%p", value, value ? value->obj : NULL);
        return;
    }

    // gpio_out is of type std_ulogic_vector(31 downto 0). To retrieve the value, we need to tell the API the type
    // that we want to retrieve (which should be compatible with the actual type), and we need to allocate memory
    // for getting the result.
    vhpiEnumT gpio_out_values[GPIO_OUT_SIZE] = {0};
    vhpiValueT gpio_out_value = {
        .format = vhpiLogicVecVal,
        .bufSize = GPIO_OUT_SIZE * sizeof(vhpiEnumT),
        .value = {
            .enumvs = gpio_out_values,
        }
    };
    int result = vhpi_get_value(value->obj, &gpio_out_value);
    if (0 != result) {
        vhpi_assert(vhpiError, "error getting gpio_out value (result=%d)");
        return;
    };
    // Use enumvs[x] to get a single value.

    // When you request a string value of a std_logic vector, you get it as a binary string. Index 0 is the left / first bit.
    unsigned char gpio_out_value_str_buf[GPIO_OUT_SIZE + 1] = {0};
    vhpiValueT gpio_out_value_str = {
        .format = vhpiStrVal,
        .bufSize = sizeof(gpio_out_value_str_buf),
        .value = {
            .str = gpio_out_value_str_buf,
        },
    };
    result = vhpi_get_value(value->obj, &gpio_out_value_str);
    if (0 != result) {
        vhpi_assert(vhpiError, "error getting gpio_out value (result=%d)");
        return;
    };
    vhpi_printf("gpio_out value changed: %s [31]=%c", (const char*) gpio_out_value_str.value.str, std_logic_value_to_str(gpio_out_value.value.enumvs[31]));
}

static void end_of_init(const vhpiCbDataT *cb_data) {
    vhpiHandleT root = vhpi_handle(vhpiRootInst, NULL);

    // Start watching gpio_out.
    vhpiHandleT gpio_out = vhpi_handle_by_name("gpio_out", root);
    if (!gpio_out) {
        vhpi_assert(vhpiError, "error getting gpio_out handle");
        return;
    }
    vhpiCbDataT cb_gpio_out_change = {
        .reason = vhpiCbValueChange,
        .cb_rtn = gpio_out_changed,
        .obj = gpio_out,
        .time = (vhpiTimeT *)-1,
    };
    if (!vhpi_register_cb(&cb_gpio_out_change, 0)) {
        vhpi_assert(vhpiError, "error registering callback for gpio_out value change!");
        return;
    }
}

static void shared_startup(void) {
    vhpi_printf("Hello world from VHPI plugin!");

    // Register callback for end of initialization, so that the signals are available and we can
    // attach watchers to them.
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
