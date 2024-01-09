//! Test what stack size is needed to clone a process that spawns some threads.

use nix::sched::CloneFlags;

// Should be default for threads, but let's just be explicit.
const THREAD_STACK_SIZE: usize = 2 * 1024 * 1024;
// Give some buffer for other stuff on the stack.
const THREAD_STACK_FOR_US: usize = THREAD_STACK_SIZE - (8 * 1024);

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let stack_size = args[1].parse().unwrap();

    // Clone with stack size specified by CLI argument.
    println!("THREAD_STACK_SIZE: {THREAD_STACK_SIZE}");
    println!("THREAD_STACK_FOR_US: {THREAD_STACK_FOR_US}");
    println!("Cloning with stack_size: {stack_size} bytes");
    let cb = Box::new(|| handle_child_process());
    let mut stack: Vec<u8> = vec![0u8; stack_size];
    match unsafe { nix::sched::clone(cb, stack.as_mut_slice(), clone_flags(), None) } {
        Ok(child) => {
            let status = nix::sys::wait::waitpid(child, None);
            println!("{:?}", status);
        }
        Err(errno) => panic!("{}", errno),
    }
}

/// Start some threads with the specified stack size and try to use the whole stack in each one.
fn handle_child_process() -> ! {
    let thread1 = std::thread::Builder::new()
        .stack_size(THREAD_STACK_SIZE)
        .spawn(|| {
            let mut arr = [0u8; THREAD_STACK_FOR_US];
            for i in 0..THREAD_STACK_FOR_US {
                arr[i] = (i % 256) as u8;
            }
            for i in 0..THREAD_STACK_FOR_US {
                assert_eq!(arr[i] as usize, i % 256);
            }
        }).unwrap();
    let thread2 = std::thread::Builder::new()
            .stack_size(THREAD_STACK_SIZE)
            .spawn(|| {
                let mut arr = [0; THREAD_STACK_FOR_US];
                for i in 0..THREAD_STACK_FOR_US {
                    arr[THREAD_STACK_FOR_US-i-1] = (i % 256) as u8;
                }
                for i in 0..THREAD_STACK_FOR_US {
                    assert_eq!(arr[THREAD_STACK_FOR_US-i-1] as usize, i % 256);
                }
            }).unwrap();

    thread1.join().unwrap();
    thread2.join().unwrap();

    std::process::exit(0);
}

/// Returns flags for `clone(2)`, including all the sandbox-related ones.
fn clone_flags() -> CloneFlags {
    // SIGCHLD flag is used to inform clone that the parent process is
    // expecting a child termination signal, without this flag `waitpid` function
    // return `ECHILD` error.
    CloneFlags::CLONE_NEWUSER
        | CloneFlags::CLONE_NEWCGROUP
        | CloneFlags::CLONE_NEWIPC
        | CloneFlags::CLONE_NEWNET
        | CloneFlags::CLONE_NEWNS
        | CloneFlags::CLONE_NEWPID
        | CloneFlags::CLONE_NEWUTS
        | CloneFlags::from_bits_retain(libc::SIGCHLD)
}
