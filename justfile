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
    hubake build .\app\rp235x-hazard3\app.kdl
    hubake pack-hex .\.work\hazard3\final\ output.hex

reboot:
    picotool reboot -u -c riscv

flash:
    openocd -f .\app\rp235x-hazard3\openocd.cfg -c "program output.hex verify reset"

entry-point:
    @echo ("IDLE   Entry Point: " + ((riscv32-unknown-elf-readelf.exe -h .\.work\hazard3\final\idle | Select-String "Entry point address") -split ":")[1].Trim())
    @echo ("SUPER  Entry Point: " + ((riscv32-unknown-elf-readelf.exe -h .\.work\hazard3\final\super | Select-String "Entry point address") -split ":")[1].Trim())
    @echo ("KERNEL Entry Point: " + ((riscv32-unknown-elf-readelf.exe -h .\.work\hazard3\final\kernel | Select-String "Entry point address") -split ":")[1].Trim())
    @echo ("BLINKY Entry Point: " + ((riscv32-unknown-elf-readelf.exe -h .\.work\hazard3\final\blinky | Select-String "Entry point address") -split ":")[1].Trim())
    
gdb:
    riscv32-unknown-elf-gdb.exe -x app/rp235x-hazard3/gdbconfig.cfg

openocd:
    openocd -f .\app\rp235x-hazard3\openocd.cfg

dump:
    riscv32-unknown-elf-objdump.exe .\.work\hazard3\final\idle -D > idle-dump.txt
    riscv32-unknown-elf-objdump.exe .\.work\hazard3\final\super -D > super-dump.txt
    riscv32-unknown-elf-objdump.exe .\.work\hazard3\final\kernel -D > kernel-dump.txt
    riscv32-unknown-elf-objdump.exe .\.work\hazard3\final\blinky -D > blinky-dump.txt

clean:
    cargo clean
    rm -r -fo .\.work
