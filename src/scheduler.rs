use cortex_m::peripheral::SYST;

use crate::supervisor::with_supervisor;

use self::task::{FramePtr, Task};

pub mod task;

// r0-r3 is already saved manually so we can use it
// to store control there so we can push it to the stack
// [r4-r7] 5*u32 - [r8-r12] 5*u32 - [default] 8*u32
// Total:

/// Returned argument is the next task to be loaded
#[no_mangle]
extern "C" fn context_switch(stack_pointer: FramePtr) -> FramePtr {
    with_supervisor(|spv| spv.sched.context_switch(stack_pointer))
}

#[export_name = "SysTick"]
pub fn system_tick() {
    with_supervisor(|sp| sp.pend_switch())
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
            // pending: Default::default(),
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

    fn context_switch(&mut self, last_stack: FramePtr) -> FramePtr {
        self.current.as_mut().unwrap().suspended_stack = last_stack;
        self.schedule_next().sp()
    }

    pub fn add_task(&mut self, task: Task) {
        self.ready.push_back(task).unwrap();
    }
}
