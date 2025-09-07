## 3.8.3.1. PMP Address Registers

Addresses in PMP address registers PMPADDR0 through PMPADDR15 are stored with a right-shift of two, so that they
can cover a 16 GB physical address space when Sv32 address translation is in effect. Hazard3 does not implement
address translation, so the physical address space is 4 GB (32-bit byte-addressed) and the two MSBs of each address
register are hardwired to zero.
The RP2350 configuration of Hazard3 supports only the OFF and NAPOT values for the PMPCFG A fields (e.g.
PMPCFG0.R0_A). Setting A to OFF means the region matches no bytes, and is effectively disabled. Setting A to NAPOT
means the region matches on a naturally aligned span of bytes (the base address modulo the size is zero) whose size is
a power of two.
The number of trailing 1s in the PMP address value encodes the size of an NAPOT region. This is the number of
consecutive 1s counted from the LSB without reaching a 0. A PMP address value with no trailing ones (ending in a 0)
matches a region eight bytes in size, and the region size is doubled with each additional 1 bit.
The PMP region matches on the address bits to the left of the least-significant 0 bit. Because the PMP address registers
are right-shifted by two, you must apply the same shift to the addresses being compared. The following examples
demonstrate how to match addresses based on PMPADDRx values:
• The 30-bit all-ones bit pattern 0x3fffffff has the maximum possible size, and matches all addresses.
• The all-zeroes bit pattern 0x00000000 has the minimum possible size.
◦ Since there are no trailing 1s, this matches starting from bit 1 of the PMP address register.
◦ Due to addresses being right-shifted by two, this is a region of eight bytes starting from address 0x0.
• The bit pattern 0x???????7 (where ? is any digit) matches any 64-byte region.
◦ Shift the base address of this 64-byte region by two to get bits 29:4 of the PMPADDRx value.
• The bit pattern 0x0800000f matches byte addresses between 0x20000000 and 0x2000007f, the first 128 bytes of SRAM.
◦ Right-shift the base address (0x20000000) by two to get 0x08000000.
◦ Add trailing ones to increase the region size and get the final value of 0x0800000f.
◦ The size of the region is eight bytes times two to the power of the number of trailing 1 bits, which in this case
(four 1s) works out to 8 × 24 = 128 bytes.
For more examples of PMP address match patterns, see the hardwired PMP region values in Section 3.8.8.1.
RP2350 configures Hazard3 with a granule of 32 bytes. This means the two least-significant bits of each PMP address
register are hardwired to all-ones when the region is enabled. The hardware does not decode address regions smaller
than 32 bytes.

## 3.8.3.2. PMP Permissions

Each 8-bit PMP configuration field contains three permission flags:
• R permits non-instruction-fetch reads:
◦ load instructions
◦ the read phase of AMOs
• W permits writes:
◦ store instructions
◦ the write phase of AMOs
• X permits instruction execution
A 1 value for each permission means it is granted, and a 0 means it is revoked. These permissions apply to U-mode
access to the region. They also apply to M-mode accesses when any of the following is true:
• The L (lock) configuration bit is 1
• The Hazard3 custom PMPCFGM0 register bit for this region is 1
The L (lock) bit also locks the associated PMP address register and 8-bit PMP configuration field, so that it ignores
future writes. You should always lock PMP regions consecutively from region 0, so that locked regions cannot be
bypassed by unlocked regions.
U-mode accesses which match no PMP regions have no permissions: all memory accesses fail. M-mode accesses
which match no PMP regions have all permissions. The hardwired PMP regions in Section 3.8.8.1 define additional Umode
permissions for the ROM and peripheral address ranges: these can be overridden by enabling any of the
dynamically configured regions.
 NOTE
Due to RP2350-E6 the field order in the PMP configuration fields is R, W, X (MSB-first) rather than the standard X, W, R.
The SDK register headers match the as-implemented order.

## 3.8.3.3. Accesses Spanning Multiple PMP Regions

Hazard3 does not support non-naturally-aligned loads or stores, other than to generate standard exceptions when they
are attempted. Since NAPOT PMP regions are always naturally aligned, it is impossible for a load or store to span two
PMP regions. Therefore all bytes covered by a load or store instruction are determined by at most a single active PMP
region which matches the lowest byte address accessed by that instruction.
Instructions are up to 32 bits in size with as little as 16-bit alignment. Therefore it is possible for an instruction to match
multiple PMP regions. When this happens, the instruction generates an instruction fault exception, (mcause = 0x1), unless
there is a lower-numbered PMP region which fully covers the instruction. Lower-numbered PMP regions take
precedence.
The exact quote from the privileged ISA specification is: "The lowest-numbered PMP entry that matches any byte of an
access determines whether that access succeeds or fails. The matching PMP entry must match all bytes of an access, or
the access fails, irrespective of the L, R, W, and X bits." (page 60 of RISC-V privileged ISA manual version 20211203).
The RISC-V specification is flexible in what is considered a single access for the purposes of memory protection
checking. Hazard3 considers the fetch of one instruction to be a single access. It therefore forbids instruction fetches
which straddle two PMP regions, even if both regions grant execute permission. Due to this architecture rule, portable
RISC-V software must not assume it can execute instructions which span multiple PMP regions.
Avoid this issue by using hole-punching region configurations in preference to glueing configurations. Suppose you want
to cover the first 12 kB of SRAM (0x20000000 → 0x20002fff), this can be achieved in two ways:

• One region adding permissions to 0x20000000 → 0x200001fff, and another region adding permissions to 0x20002000 →
0x20002fff
• One region adding permissions to 0x20000000 → 0x20003fff, and a lower-numbered region subtracting permissions
from 0x20003000 → 0x20003fff
The former option has a crack between the two regions, which has potentially unwanted effects on some platforms. The
latter avoids this issue entirely.

## 3.8.8.1. Hardwired PMP Regions

RP2350 configures Hazard3 with eight dynamically configured PMP regions, and three static ones. The static regions
provide default U-mode RWX permissions on the following ranges:
• ROM: 0x00000000 through 0x0fffffff
• Peripherals: 0x40000000 through 0x5fffffff
• SIO: 0xd0000000 through 0xdfffffff
These addresses appear in PMPADDR8, PMPADDR9 and PMPADDR10. The hardwired PMP address registers behave
the same as dynamic registers, except that they ignore writes (exercising the WARL rule). The permissions for these
regions are in PMPCFG2.
The hardwired regions have a similar role to the Exempt regions added to the Cortex-M33 IDAU address map specified
in Section 10.2.2.
RP2350 puts default U-mode permissions on AHB/APB peripherals because these are expected to be assigned using
ACCESSCTRL (Section 10.6). ACCESSCTRL can assign each peripheral individually, using the existing address decoders
in the bus fabric, whereas PMP regions are in limited supply so are less useful for peripheral assignment.
Similarly, SIO has internal banking over Secure/Non-secure bus attribution, which is mapped onto Machine and User
modes as described in Section 10.6.2.
The dynamic regions 0 through 7 take priority over the hardwired regions, because the PMP prioritises lower-numbered
regions.
From GDB, i got the values stored for hardware PMP regions:
pmpcfg2        0x1f1f1f         2039583
pmpaddr8       0x1ffffff        33554431
pmpaddr9       0x13ffffff       335544319
pmpaddr10      0x35ffffff       905969663


## RP2350-E6

Reference RP2350-E6
Summary PMPCFGx RWX fields are transposed
Affects RP2350 A2
Description The Physical Memory Protection unit (PMP) defines read, write and execute permissions (RWX) for
configurable ranges of physical memory. The RWX permissions for four regions are packed into each 32-
bit PMPCFG register, PMPCFG0 through PMPCFG3.
Per the RISC-V privileged ISA specification, the permission fields are ordered X, W, R from MSB to LSB.
Hazard3 implements them in the order R, W, X. This means software using the correct bit order will have
its read permissions applied as execute, and vice versa. (See upstream commit 7d37029.)
Workaround When configuring PMP with X != R, use the bit order implemented by this version of Hazard3. In the SDK,
the hardware/regs/rvcsr.h register header provides bitfield definitions for the as-implemented order when
building for RP2350.
Fixed by Documentation

