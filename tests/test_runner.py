import os
from pathlib import Path

from cocotb_tools.runner import get_runner

def test_runner():
    sim = os.getenv("SIM", "nvc")

    neorv32_home = Path(__file__).resolve().parent.parent / "neorv32"

    sources = [
        str(neorv32_home / "rtl" / "test_setups" / "neorv32_test_setup_approm.vhd")
    ]

    runner = get_runner(sim)
    runner.build(
        sources=sources,
        hdl_toplevel="neorv32_test_setup_approm",
        # Note: it is assumed that the neorv32 library has been compiled using the Makefile.
        build_args=[],
    )

    runner.test(hdl_toplevel="neorv32_test_setup_approm", test_module="test_hello_world,",
                )


if __name__ == "__main__":
    test_runner()
