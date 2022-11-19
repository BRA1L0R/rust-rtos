use crate::supervisor::with_supervisor;

use self::task::{ExtendedFrame, Task};

pub mod task;

// r0-r3 is already saved manually so we can use it
// to store control there so we can push it to the stack
// [r4-r7] 5*u32 - [r8-r12] 5*u32 - [default] 8*u32
// Total:

#[no_mangle]
extern "C" fn pendsv(stack_pointer: *mut ExtendedFrame) -> *mut ExtendedFrame {
    with_supervisor(|spv| spv.sched.context_switch(stack_pointer))
}

#[derive(Default)]
pub(crate) struct Scheduler {
    tasks: heapless::Vec<Task, 10>,
    last: usize,
}

impl Scheduler {
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

    fn context_switch(&mut self, last_stack: *mut ExtendedFrame) -> *mut ExtendedFrame {
        self.current_mut().suspended_stack = last_stack;
        self.schedule_next().sp()
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task).unwrap();
    }
}
