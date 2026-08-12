import cocotb
from cocotb.triggers import Timer
from cocotb.clock import Clock

@cocotb.test()
async def test_blinky_led(dut):
    clock = Clock(dut.clk_i.value, 100, "ns")  # 100 MHz

    await Timer(1, unit="ms")
    cocotb.log.info("1 ms has passed, output is %s", dut.gpio_o[0].value)

    await Timer(100, unit="ms")
    cocotb.log.info("100 ms has passed, output is %s", dut.gpio_o[0].value)

    await Timer(100, unit="ms")
    cocotb.log.info("100 ms has passed, output is %s", dut.gpio_o[0].value)

    await Timer(100, unit="ms")
    cocotb.log.info("100 ms has passed, output is %s", dut.gpio_o[0].value)