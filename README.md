# neorv32-test

Test project for checking out neorv32.

## Quick start

In `Makefile`, select the example to use.

```
# Define here which example program is installed in the imem
# neorv32/rtl/core/neorv32_imem_image.vhd: hello_world
neorv32/rtl/core/neorv32_imem_image.vhd: blink_led
```

Then run with

```
make run
```

To run the test suite with cocotb, just run

```
make sim_build/neorv32
uv run python3 tests/test_runner.py
```

## How the examples work

Go to a particular example:

```
cd neorv32/sw/example/hello_world
```

Compile the image into a local file and install it into `neorv32_imem_image.vhd`:

```
make USER_FLAGS+=-DUART0_SIM_MODE clean image install
```
