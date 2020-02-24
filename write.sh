#!/bin/sh

cargo objcopy --bin nrf-blinky -- -O ihex nrf-blinky.hex && \
    adafruit-nrfutil dfu genpkg --sd-req 0xFFFE --dev-type 0x0052 --application nrf-blinky.hex  dfu-package.zip && \
    adafruit-nrfutil --verbose dfu serial --package dfu-package.zip -p /dev/ttyACM0 -b 115200 --singlebank --touch 1200
