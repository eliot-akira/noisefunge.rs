
use rand::Rng;
use std::rc::Rc;

use super::process::{Process, Prog, ProcessState, Syscall, Dir, Op, PC};

macro_rules! pop {
    ($proc : ident) => {
        match $proc.pop() {
            Some(u) => u,
            None => {
                $proc.die("Pop from empty stack.");
                return
            }
        }
    }
}

pub struct OpSet([Option<Op>; 256]);

impl OpSet {
    pub fn new() -> OpSet {
        let mut ops : [Option<Op>; 256] = [
            // I guess some 3rd party crates solve this problem...
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
            None, None, None, None, None, None, None, None,
        ];
        ops[32] = Some(Op::new(Box::new(noop))); // Space
        ops[60] = Some(set_direction(Dir::L)); // >
        ops[62] = Some(set_direction(Dir::R)); // <
        ops[94] = Some(set_direction(Dir::U)); // ^
        ops[118] = Some(set_direction(Dir::D)); // v
        ops[63] = Some(Op::new(Box::new(rand_direction))); // ?
        ops[59] = Some(Op::new(Box::new(r#return))); // ;
        ops[64] = Some(Op::new(Box::new(quit))); // @

        for i in 0..=9 { // 0 - 9
            ops[i as usize + 48] = Some(push_int(i));
        }
        for i in 0..=5 { // A - F
            ops[i as usize + 65] = Some(push_int(10 + i));
        }

        ops[37] = Some(Op::new(Box::new(r#mod))); // %
        ops[42] = Some(Op::new(Box::new(mul))); // *
        ops[43] = Some(Op::new(Box::new(add))); // +
        ops[45] = Some(Op::new(Box::new(sub))); // -
        ops[47] = Some(Op::new(Box::new(div))); // /

        OpSet(ops)
    }

    pub fn apply_to(&self, proc: &mut Process) {
        let OpSet(ops) = self;
        let c = match proc.peek() {
            None => return,
            Some(c) => c
        };
        match &ops[c as usize] {
            None => return,
            Some(op) => proc.apply(op)
        }
    }

}

fn noop(proc: &mut Process) {
    proc.step()
}

fn push_int(i: u8) -> Op {
    let push_i = move |proc: &mut Process| {
        proc.push(i)
    };
    Op::new(Box::new(push_i))
}

fn set_direction(dir: Dir) -> Op {
    let set_dir = move |proc: &mut Process| {
        proc.set_direction(dir);
        proc.step()
    };
    Op::new(Box::new(set_dir))
}

fn rand_direction(proc: &mut Process) {
    let mut rng = rand::thread_rng();
    let dir = match rng.gen_range(0,4) {
        0 => Dir::L,
        1 => Dir::R,
        2 => Dir::U,
        3 => Dir::D,
        _ => panic!("Random number out of range [0,4)")
    };
    proc.set_direction(dir);
    proc.step();
}

fn sleep(proc: &mut Process) {
    let beats = pop!(proc);
    proc.trap(Syscall::Sleep(beats));
}
 
fn r#return(proc: &mut Process) {
    proc.r#return();
}

fn quit(proc: &mut Process) {
    proc.set_state(ProcessState::Finished);
}

fn add(proc: &mut Process) {
    let x = pop!(proc);
    let y = pop!(proc);
    proc.push(x + y);
    proc.step();
}

fn sub(proc: &mut Process) {
    let x = pop!(proc);
    let y = pop!(proc);
    proc.push(y - x);
    proc.step();
}

fn mul(proc: &mut Process) {
    let x = pop!(proc);
    let y = pop!(proc);
    proc.push(x * y);
    proc.step();
}

fn div(proc: &mut Process) {
    let x = pop!(proc);
    let y = pop!(proc);
    proc.push(y / x);
    proc.step();
}

fn r#mod(proc: &mut Process) {
    let x = pop!(proc);
    let y = pop!(proc);
    proc.push(y % x);
    proc.step();
}

fn fork(proc: &mut Process) {
    proc.trap(Syscall::Fork)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn demo_proc() -> Process {
        Process::new(1, Rc::from("a"), Rc::from("b"),
            Prog::parse(">    @\n\
                         > 1 2^\n\
                         >45+ ^\n\
                         >95- ^\n\
                         >35* ^\n\
                         >48/ ^\n\
                         >A7% ^").expect("Bad test program."))
    }

    #[test]
    fn test_noop() {
        let mut proc = demo_proc();
        proc.top_mut().map(|t| t.pc = PC(1));
        let ops = OpSet::new();
        ops.apply_to(&mut proc);
        let PC(i) = proc.top().expect("Empty top").pc;
        assert!(i == 2, "PC != 2");
        assert!(proc.state() == ProcessState::Running(false),
                "Process is not running.");

        // Rest of program plays out.
        for _ in 1..10 {
            ops.apply_to(&mut proc);
        }
        assert!(proc.state() == ProcessState::Finished,
                "Process is not running.");
    }

}