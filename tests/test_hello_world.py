import cocotb
from cocotb.triggers import Timer
from cocotb.clock import Clock

import logging
log = logging.getLogger("test")

@cocotb.test()
async def test_blinky_led(dut):
    await Timer(1, unit="ms")
    log.info("1 ms has passed, output is %s", dut.gpio_out.value[0])

    await Timer(100, unit="ms")
    log.info("100 ms has passed, output is %s", dut.gpio_out.value[0])

    await Timer(100, unit="ms")
    log.info("100 ms has passed, output is %s", dut.gpio_out.value[0])

    await Timer(100, unit="ms")
    log.info("100 ms has passed, output is %s", dut.gpio_out.value[0])
