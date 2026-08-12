import cocotb
from cocotb.triggers import Timer
from cocotb.clock import Clock

import logging
log = logging.getLogger("test")

@cocotb.test()
async def test_blinky_led(dut):
    clock = Clock(dut.clk_i, 100, "ns")  # 100 MHz
    clock.start()

    await Timer(1, unit="ms")
    log.info("1 ms has passed, output is %s", dut.gpio_o.value[0])

    await Timer(100, unit="ms")
    log.info("100 ms has passed, output is %s", dut.gpio_o.value[0])

    await Timer(100, unit="ms")
    log.info("100 ms has passed, output is %s", dut.gpio_o.value[0])

    await Timer(100, unit="ms")
    log.info("100 ms has passed, output is %s", dut.gpio_o.value[0])
