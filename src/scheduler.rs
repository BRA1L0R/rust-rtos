use crate::supervisor;

use self::task::{FramePtr, PendingTask, Task};
use cortex_m::{interrupt::CriticalSection, peripheral::SYST};

pub mod task;

// r0-r3 is already saved manually so we can use it
// to store control there so we can push it to the stack
// [r4-r7] 5*u32 - [r8-r12] 5*u32 - [default] 8*u32
// Total:

/// Returned argument is the next task to be loaded
#[no_mangle]
extern "C" fn context_switch(stack_pointer: FramePtr) -> FramePtr {
    // Safety: all interrupts have same priority
    let cs = unsafe { CriticalSection::new() };

    let mut spv = supervisor::supervisor(&cs);
    spv.sched.context_switch(stack_pointer)
}

#[no_mangle]
extern "C" fn system_tick() {
    // Safety: all interrupts have same priority
    let cs = unsafe { CriticalSection::new() };
    supervisor::supervisor(&cs).pend_switch();
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
        const TICK_SPEED: u32 = 0xFFFF;

        self.systick.enable_interrupt();
        self.systick.set_reload(TICK_SPEED);
        self.systick.clear_current();
        self.systick.enable_counter();
    }

    pub fn schedule_next(&mut self) -> &Task {
        if let Some(task) = self.current.take() {
            self.ready.push_back(task).unwrap();
        }

        let task = self
            .ready
            .pop_front()
            .expect("cannot run without ready tasks yet");

        self.current.insert(task)
    }

    pub fn pend_current(&mut self) -> PendingTask {
        let task = self
            .current
            .take()
            .expect("called pend without any task running");

        PendingTask::new(task)
    }

    fn context_switch(&mut self, last_stack: FramePtr) -> FramePtr {
        self.current.as_mut().unwrap().suspended_stack = last_stack;
        self.schedule_next().sp()
    }

    pub fn schedule_task(&mut self, task: Task) {
        self.ready.push_back(task).unwrap();
    }
}
