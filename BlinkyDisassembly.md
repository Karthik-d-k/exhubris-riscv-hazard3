## Blinky Disassembly


```rs
// Set GPIO as output
peripherals
    .sio
    .gpio_oe_set()
    .write(|w| unsafe { w.bits(mask) });
```
-------------------------------------------------------------------------------------

`10000350 <main>:`

> a0 <= SIO_BASE = 0xD000_0000

`10000350:	d0000537          	lui	a0,0xd0000`

> a1 <= 0x0040_0000 // set 22nd bit

`10000354:	004005b7          	lui	a1,0x400`

> 56(a0) = 0x038(a0) = SIO->GPIO_OE_SET <= a1

`10000358:	dd0c               	sw	a1,56(a0)`

-------------------------------------------------------------------------------------

```rs
// Configure pad settings
peripherals.pads_bank0.gpio(led_pin).modify(|_, w| {
    // Set input enable on, output disable off
    // RP2350: input enable defaults to off, so this is important!
    w.ie().set_bit();
    w.od().clear_bit();
    // RP2350: remove pad isolation now a function is wired up
    w.iso().clear_bit();
    w
});
```
-------------------------------------------------------------------------------------

> a0 <= PADS_BANK0_BASE = 0x4003_8000

`1000035a:	40038537          	lui	a0,0x40038`

> a1 <= 92(a0) = 5C(a0) = PADS_BANK0->GPIO22

`1000035e:	4d6c                lw	a1,92(a0)`

> Set 6th bit (IE)

> Clear 7th and 8th bits (OD, ISO)

`10000360:	e3f5f593          	and	a1,a1,-449`

`10000364:	04058593          	add	a1,a1,64 # 400040`

`10000368:	cd6c               	sw	a1,92(a0)`

-------------------------------------------------------------------------------------

```rs
// Zero all fields apart from fsel; we want this IO to do what the peripheral tells it.
// This doesn't affect e.g. pullup/pulldown, as these are in pad controls.
unsafe {
    peripherals
        .io_bank0
        .gpio(led_pin)
        .gpio_ctrl()
        .write_with_zero(|w| w.funcsel().variant(FUNCSEL_A::SIO));
};
```

-------------------------------------------------------------------------------------

> a0 <= IO_BANK0_BASE = 0x4002_8000

`1000036a:	40028537          	lui	a0,0x40028`

> a1 <= 5

`1000036e:	4595                li	a1,5`

> Store 0x05(SIO_22) at IO_BANK0_BASE->GPIO22_CTRL

`10000370:	0ab52a23          	sw	a1,180(a0) # 400280b4`

`10000374:	a001                j	10000374 <main+0x24>`

-------------------------------------------------------------------------------------

