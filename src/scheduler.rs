use cortex_m::peripheral::SYST;

use crate::supervisor::with_supervisor;

use self::task::{Task, TaskFrame};

pub mod task;

// r0-r3 is already saved manually so we can use it
// to store control there so we can push it to the stack
// [r4-r7] 5*u32 - [r8-r12] 5*u32 - [default] 8*u32
// Total:

/// Returned argument is the next task to be loaded
#[no_mangle]
extern "C" fn context_switch(stack_pointer: TaskFrame) -> TaskFrame {
    with_supervisor(|spv| spv.sched.context_switch(stack_pointer))
}

#[export_name = "SysTick"]
pub fn system_tick() {
    with_supervisor(|sp| sp.pend_switch())
}

// static SCHEDULER: Mutex<RefCell<Scheduler>> = Mutex::new(RefCell::new)

// #[derive(Default)]
pub struct Scheduler {
    systick: SYST,

    tasks: heapless::Vec<Task, 10>,
    last: usize,
}

impl Scheduler {
    pub fn new(systick: SYST) -> Self {
        Self {
            systick,
            tasks: Default::default(),
            last: 0,
        }
    }

    pub fn start_systick(&mut self) {
        const TICK_SPEED: u32 = 0xFFFF;

        self.systick.enable_interrupt();
        self.systick.set_reload(TICK_SPEED);
        self.systick.clear_current();
        self.systick.enable_counter();
    }

    pub(crate) fn current(&self) -> &Task {
        &self.tasks[self.last]
    }

    fn current_mut(&mut self) -> &mut Task {
        &mut self.tasks[self.last]
    }

    fn schedule_next(&mut self) -> &Task {
        self.last = (self.last + 1) % self.tasks.len();
        &self.tasks[self.last]
    }

    fn context_switch(&mut self, last_stack: TaskFrame) -> TaskFrame {
        self.current_mut().suspended_stack = last_stack;
        self.schedule_next().sp()
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task).unwrap();
    }
}
