//! Drives the device through the real `BusPeripheral` with a scripted bus master.

use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use device_audio::core::{reg, Policy};
use device_audio::new_audio_device;
use peripheral_bus::device::{BusAssertions, BusOperation, Device, LEVEL_THREE_INTERRUPT};
use peripheral_bus::new_bus_peripheral;

const BASE: u32 = 0x0080_0000;

type Results = Rc<RefCell<Vec<(u16, u64)>>>;

struct Master {
    script: Vec<(u32, bool, u16)>,
    idx: usize,
    waiting: bool,
    started: u64,
    now: u64,
    gap: u64,
    results: Results,
}

impl Device for Master {
    fn poll(&mut self, bus: BusAssertions, _selected: bool) -> BusAssertions {
        self.now += 1;
        if self.waiting && bus.bus_acknowledge {
            self.results
                .borrow_mut()
                .push((bus.data, self.now - self.started));
            self.idx += 1;
            self.waiting = false;
            self.gap = 3; // bus idle between accesses, like a real CPU
            return BusAssertions::default();
        }
        if self.gap > 0 {
            self.gap -= 1;
            return BusAssertions::default();
        }
        if self.idx < self.script.len() {
            if !self.waiting {
                self.waiting = true;
                self.started = self.now;
            }
            let (address, write, data) = self.script[self.idx];
            return BusAssertions {
                address,
                data,
                op: if write {
                    BusOperation::Write
                } else {
                    BusOperation::Read
                },
                bus_access_strobe: true,
                ..BusAssertions::default()
            };
        }
        BusAssertions::default()
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

#[test]
fn register_access_acks_at_once_and_ram_waits_for_cpu_slots() {
    let results: Results = Rc::default();
    let script = vec![
        (BASE + u32::from(reg::MASTER_VOL), true, 0x0042),
        (BASE + u32::from(reg::MASTER_VOL), false, 0),
        (BASE + 0x8000 + 0x0123, true, 0xBEEF),
        (BASE + 0x8000 + 0x0123, false, 0),
        (BASE + 0x0100, false, 0), // reserved: reads zero
        (BASE + u32::from(reg::STATUS), false, 0),
    ];
    let n = script.len();
    let master = Master {
        script,
        idx: 0,
        waiting: false,
        started: 0,
        now: 0,
        gap: 0,
        results: results.clone(),
    };
    let mut bus = new_bus_peripheral(Box::new(master));
    bus.map_segment(
        "AUDIO",
        BASE,
        0x10000,
        true,
        Box::new(new_audio_device(Policy::default())),
    );

    let mut a = BusAssertions::default();
    for _ in 0..400 {
        a = bus.poll_all(a);
    }
    let r = results.borrow();
    assert_eq!(r.len(), n);
    assert_eq!(r[1].0, 0x0042, "register readback");
    assert_eq!(
        r[0].1, 1,
        "register write acknowledged without waiting for a RAM slot"
    );
    assert_eq!(r[1].1, 1);
    assert_eq!(r[3].0, 0xBEEF, "RAM readback through the CPU window");
    assert!(
        (2..=6).contains(&r[2].1) && (2..=6).contains(&r[3].1),
        "RAM latency {:?}",
        (r[2].1, r[3].1)
    );
    assert_eq!(r[4].0, 0);
    assert_eq!(r[5].0, 1, "STATUS reads revision 1, no flags");
    println!(
        "bus latencies (polls from strobe to BACK): {:?}",
        r.iter().map(|x| x.1).collect::<Vec<_>>()
    );
}

#[test]
fn irq3_is_asserted_on_the_bus_while_completion_is_enabled() {
    let mut dev = new_audio_device(Policy::default());
    dev.core.reg_write(reg::SEQ_DONE, 0); // no-op
                                          // Park a sequencer on END 0 using the backdoor, then let it run.
    dev.core
        .set_ram_word(0x100, device_audio::testkit::isa::end(0));
    dev.core.reg_write(reg::SEQ_PERIOD, 2);
    dev.core
        .reg_write(reg::ch(0, reg::CH_CONTROL), reg::CONTROL_SEQ_EN);
    dev.core.reg_write(reg::ch(0, reg::CH_SEQ_PC), 0x8100);
    dev.core.reg_write(reg::SEQ_INT_EN, 1);
    let mut bus = new_bus_peripheral(Box::new(Master {
        script: vec![],
        idx: 0,
        waiting: false,
        started: 0,
        now: 0,
        gap: 0,
        results: Rc::default(),
    }));
    bus.map_segment("AUDIO", BASE, 0x10000, true, Box::new(dev));
    let mut a = BusAssertions::default();
    let mut seen = false;
    for _ in 0..336 * 8 {
        a = bus.poll_all(a);
        seen |= a.interrupt_assertion & LEVEL_THREE_INTERRUPT != 0;
    }
    assert!(seen, "IRQ3 should be raised once the sequence completes");
}
