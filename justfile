# use PowerShell instead of sh:
set shell := ["powershell.exe", "-c"]

alias b := build
alias r := reboot
alias f := flash
alias g := gdb

default:
    @just --list

build:
    hubake build .\app\demo-hazard3\app.kdl
    hubake pack-hex .\.work\hazard3\final\ output.hex

reboot:
    picotool reboot -u -c riscv

flash:
    openocd -f .\app\demo-hazard3\openocd.cfg -c "program output.hex verify"

gdb:
    riscv32-unknown-elf-gdb.exe -x app/demo-hazard3/gdbconfig.cfg
