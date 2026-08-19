#!/usr/bin/env python3

import os
from pathlib import Path

from cocotb_tools.runner import get_runner

def test_runner():
    sim = os.getenv("SIM", "nvc")

    neorv32_home = Path(__file__).resolve().parent.parent / "neorv32"

    lib_sources = (neorv32_home / "rtl" / "file_list_core.f").read_text().splitlines()
    sources = lib_sources + [
        str(neorv32_home / "sim" / f)
        for f in [
            "psram_model.vhd",
            "sim_uart_rx.vhd",
            "xbus_fmem.vhd",
            "xbus_gateway.vhd",
            "xbus_memory.vhd",
            "jtag_dmi_pkg.vhd",
            "neorv32_tb.vhd",
        ]
    ]

    runner = get_runner(sim)
    runner.build(
        sources=sources,
        hdl_toplevel="neorv32_tb",
        # Note: it is assumed that the neorv32 library has been compiled using the Makefile.
        build_args=[],
        verbose=True,
    )

    runner.test(hdl_toplevel="neorv32_tb", test_module="test_hello_world,",
                test_args=["--ieee-warnings=off"], waves=True, verbose=True
                )


if __name__ == "__main__":
    test_runner()
