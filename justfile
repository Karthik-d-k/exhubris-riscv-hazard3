# use PowerShell instead of sh:
set shell := ["powershell.exe", "-c"]

# Common aliases
alias b := build
alias r := reboot
alias f := flash
alias g := gdb
alias d := dump
alias e := entry-point

default:
    @just --list

# ARM recipes
build:
    hubake build app\cortex-m33\app.kdl
    hubake pack-hex .\.work\cortex-m33\final\ output.hex

reboot:
    picotool reboot -u -c arm

flash:
    openocd -f app/cortex-m33/openocd.cfg -c "program output.hex verify"

entry-point:
    @echo ("IDLE   Entry Point: " + ((arm-none-eabi-readelf.exe -h .\.work\cortex-m33\final\idle | Select-String "Entry point address") -split ":")[1].Trim())
    @echo ("SUPER  Entry Point: " + ((arm-none-eabi-readelf.exe -h .\.work\cortex-m33\final\super | Select-String "Entry point address") -split ":")[1].Trim())
    @echo ("BLINKY Entry Point: " + ((arm-none-eabi-readelf.exe -h .\.work\cortex-m33\final\blinky | Select-String "Entry point address") -split ":")[1].Trim())
    @echo ("KERNEL Entry Point: " + ((arm-none-eabi-readelf.exe -h .\.work\cortex-m33\final\kernel | Select-String "Entry point address") -split ":")[1].Trim())
    
gdb: entry-point
    arm-none-eabi-gdb.exe -x app/cortex-m33/gdbconfig.cfg

dump:
    arm-none-eabi-objdump.exe -D .\.work\cortex-m33\final\idle > idle-dump-arm.txt
    arm-none-eabi-objdump.exe -D .\.work\cortex-m33\final\super > super-dump-arm.txt
    arm-none-eabi-objdump.exe -D .\.work\cortex-m33\final\blinky > blinky-dump-arm.txt
    arm-none-eabi-objdump.exe -D .\.work\cortex-m33\final\kernel > kernel-dump-arm.txt

