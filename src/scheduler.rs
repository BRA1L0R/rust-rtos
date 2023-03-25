use crate::{arch::switching, supervisor};

use self::{
    handle::TaskHandle,
    task::{FramePtr, SuspendedTask, Task},
};
use cortex_m::{interrupt::CriticalSection, peripheral::SYST};

pub mod arguments;
mod handle;
pub mod task;

// r0-r3 is already saved manually so we can use it
// to store control there so we can push it to the stack
// [r4-r7] 5*u32 - [r8-r12] 5*u32 - [default] 8*u32
// Total:

/// Returned argument is the next task to be loaded
#[no_mangle]
extern "C" fn context_switch() -> FramePtr {
    // Safety: all interrupts have same priority
    let cs = unsafe { CriticalSection::new() };

    let mut spv = supervisor::supervisor(&cs);
    spv.sched.schedule_next()
}

#[no_mangle]
extern "C" fn system_tick() {
    // Safety: all interrupts have same priority
    let cs = unsafe { CriticalSection::new() };

    // Safety: systemtick is called right after
    // and only after saving a frame
    let frame = unsafe { switching::current_extended() };
    let mut spv = supervisor::supervisor(&cs);

    spv.sched.save_current(frame);
    spv.pend_switch();
}

// #[derive(Default)]
pub struct Scheduler {
    systick: SYST,

    ready: heapless::Deque<Task, 10>,
    current: Option<Task>,
}

impl Scheduler {
    pub fn new(systick: SYST) -> Self {
        Self {
            systick,
            ready: Default::default(),
            current: None,
        }
    }

    pub fn start_systick(&mut self) {
        const TICK_SPEED: u32 = 0x1FFF;

        self.systick.enable_interrupt();
        self.systick.set_reload(TICK_SPEED);
        self.systick.clear_current();
        self.systick.enable_counter();
    }

    pub fn schedule_next(&mut self) -> FramePtr {
        if let Some(task) = self.current.take() {
            self.ready.push_back(task).unwrap();
        }

        let task = self
            .ready
            .pop_front()
            .expect("cannot run without ready tasks yet");

        self.current.insert(task).sp()
    }

    // pub fn suspend_current(&mut self) -> SuspendedTask {
    //     let task = self
    //         .current
    //         .take()
    //         .expect("called suspend without any task running");

    //     SuspendedTask::new(task)
    // }

    pub fn save_current(&mut self, frame: FramePtr) -> TaskHandle<'_> {
        let mut task = TaskHandle::new(&mut self.current).unwrap();
        task.save_sp(frame);

        task
    }

    pub fn schedule_task(&mut self, task: Task) {
        self.ready.push_back(task).unwrap();
    }
}
