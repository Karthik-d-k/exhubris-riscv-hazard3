# use PowerShell instead of sh:
set shell := ["powershell.exe", "-c"]

# Common aliases
alias b := build-riscv
alias r := reboot-riscv
alias f := flash-riscv
alias g := gdb-riscv
alias d := dump-riscv
alias e := entry-point-riscv

# ARM aliases
alias ba := build-arm
alias ra := reboot-arm
alias fa := flash-arm
alias ga := gdb-arm
alias da := dump-arm
alias ea := entry-point-arm

default:
    @just --list

# RISC-V recipes
build-riscv:
    hubake build .\app\demo-hazard3\app.kdl
    hubake pack-hex .\.work\hazard3\final\ output.hex

reboot-riscv:
    picotool reboot -u -c riscv

flash-riscv:
    openocd -f .\app\demo-hazard3\openocd.cfg -c "program output.hex verify"

entry-point-riscv:
    @echo ("IDLE   Entry Point: " + ((riscv32-unknown-elf-readelf.exe -h .\.work\hazard3\final\idle | Select-String "Entry point address") -split ":")[1].Trim())
    @echo ("SUPER  Entry Point: " + ((riscv32-unknown-elf-readelf.exe -h .\.work\hazard3\final\super | Select-String "Entry point address") -split ":")[1].Trim())
    @echo ("KERNEL Entry Point: " + ((riscv32-unknown-elf-readelf.exe -h .\.work\hazard3\final\kernel | Select-String "Entry point address") -split ":")[1].Trim())
    @echo ("BLINKY Entry Point: " + ((riscv32-unknown-elf-readelf.exe -h .\.work\hazard3\final\blinky | Select-String "Entry point address") -split ":")[1].Trim())
    
gdb-riscv:
    riscv32-unknown-elf-gdb.exe -x app/demo-hazard3/gdbconfig.cfg

dump-riscv:
    riscv32-unknown-elf-objdump.exe .\.work\hazard3\final\idle -D > idle-dump.txt
    riscv32-unknown-elf-objdump.exe .\.work\hazard3\final\super -D > super-dump.txt
    riscv32-unknown-elf-objdump.exe .\.work\hazard3\final\kernel -D > kernel-dump.txt
    riscv32-unknown-elf-objdump.exe .\.work\hazard3\final\blinky -D > blinky-dump.txt

# ARM recipes
build-arm:
    hubake build .\app\demo-cortex-m33\app.kdl
    hubake pack-hex .\.work\cortex-m33\final\ output-arm.hex

reboot-arm:
    picotool reboot -u -c arm

flash-arm:
    openocd -f .\app\demo-cortex-m33\openocd.cfg -c "program output-arm.hex verify"

entry-point-arm:
    @echo ("IDLE   Entry Point: " + ((arm-none-eabi-readelf.exe -h .\.work\cortex-m33\final\idle | Select-String "Entry point address") -split ":")[1].Trim())
    @echo ("SUPER  Entry Point: " + ((arm-none-eabi-readelf.exe -h .\.work\cortex-m33\final\super | Select-String "Entry point address") -split ":")[1].Trim())
    @echo ("KERNEL Entry Point: " + ((arm-none-eabi-readelf.exe -h .\.work\cortex-m33\final\kernel | Select-String "Entry point address") -split ":")[1].Trim())
    @echo ("BLINKY Entry Point: " + ((arm-none-eabi-readelf.exe -h .\.work\cortex-m33\final\blinky | Select-String "Entry point address") -split ":")[1].Trim())
    
gdb-arm:
    arm-none-eabi-gdb.exe -x app/demo-cortex-m33/gdbconfig.cfg

dump-arm:
    arm-none-eabi-objdump.exe .\.work\cortex-m33\final\kernel -D > kernel-dump-arm.txt
    arm-none-eabi-objdump.exe .\.work\cortex-m33\final\idle -D > idle-dump-arm.txt
    arm-none-eabi-objdump.exe .\.work\cortex-m33\final\super -D > super-dump-arm.txt
    arm-none-eabi-objdump.exe .\.work\cortex-m33\final\blinky -D > blinky-dump-arm.txt
