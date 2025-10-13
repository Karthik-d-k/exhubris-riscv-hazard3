# use PowerShell instead of sh:
set shell := ["powershell.exe", "-c"]

# Common aliases
alias b := build
alias r := reboot
alias f := flash
alias g := gdb
alias o := openocd
alias d := dump
alias e := entry-point
alias c := clean

default:
    @just --list

# RISC-V recipes
build:
    hubake build .\app\demo-gpio-led-blink\app.kdl
    hubake pack-hex .\.work\demo-gpio-led-blink\final\ output.hex

reboot:
    picotool reboot -u -c riscv

flash:
    openocd -f .\app\demo-gpio-led-blink\openocd.cfg -c "program output.hex verify reset"

entry-point:
    @echo ("KERNEL Entry Point: " + ((riscv32-unknown-elf-readelf.exe -h .\.work\demo-gpio-led-blink\final\kernel | Select-String "Entry point address") -split ":")[1].Trim())
    @echo ("IDLE   Entry Point: " + ((riscv32-unknown-elf-readelf.exe -h .\.work\demo-gpio-led-blink\final\idle | Select-String "Entry point address") -split ":")[1].Trim())
    @echo ("SUPER  Entry Point: " + ((riscv32-unknown-elf-readelf.exe -h .\.work\demo-gpio-led-blink\final\super | Select-String "Entry point address") -split ":")[1].Trim())
    @echo ("BLINKY Entry Point: " + ((riscv32-unknown-elf-readelf.exe -h .\.work\demo-gpio-led-blink\final\blinky | Select-String "Entry point address") -split ":")[1].Trim())

gdb:
    riscv32-unknown-elf-gdb.exe -q -x app/demo-gpio-led-blink/gdbconfig.cfg

openocd:
    openocd -f .\app\demo-gpio-led-blink\openocd.cfg

dump:
    riscv32-unknown-elf-objdump.exe .\.work\demo-gpio-led-blink\final\kernel -D > kernel-dump.txt
    riscv32-unknown-elf-objdump.exe .\.work\demo-gpio-led-blink\final\idle -D > idle-dump.txt
    riscv32-unknown-elf-objdump.exe .\.work\demo-gpio-led-blink\final\super -D > super-dump.txt
    riscv32-unknown-elf-objdump.exe .\.work\demo-gpio-led-blink\final\blinky -D > blinky-dump.txt

clean:
    cargo clean
    rm -r -fo .\.work
