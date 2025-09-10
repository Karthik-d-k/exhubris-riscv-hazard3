//! "Fake" syscall wrapper module.
//!
//! This exists for two reasons:
//!
//! 1. To provide architecture-neutral syscall docs for `cargo doc`.
//! 2. To make `userlib` compile on other platforms for `rust-analyzer`.

use crate::{
    AbiLease, Lease, Message, MessageOrNotification, ReplyFaultReason, ResponseCode, Sysnum,
    TaskDeath, TaskId, TimerSettings,
};
use core::arch::global_asm;
use core::mem::MaybeUninit;

extern "Rust" {
    /// Unresolved symbol for the application main function.
    fn main() -> !;
}

extern "C" {
    /// Our actual startup routine.
    fn _start() -> !;
}

pub(crate) fn idle() {
    riscv::asm::wfi();
}

/// Stops the current task with a panic message.
///
/// This induces a fault in the current task, causing a notification to the
/// supervisor. The panic message provided is available to the supervisor and
/// debug tools, but is not interpreted by the kernel itself. The message should
/// normally be UTF-8 but, in practice, may contain a truncated final UTF-8 byte
/// sequence due to space constraints.
pub fn sys_panic(msg: &[u8]) -> ! {
    let _ = msg;
    unimplemented!();
}

/// Sends a function-call-style synchronous IPC, blocking for a response.
///
/// This sends a message to the task identified by `target`. The message is
/// marked with an operation code (`operation`) and carries an arbitrary byte
/// payload (`outgoing`). When `target` decides to reply, the reply will be
/// written into a buffer set aside for the purpose (`incoming`).
///
/// While the IPC is outstanding, the caller is granted access to a set of
/// regions of memory in the calling task (`leases`) through the borrow
/// syscalls.
///
/// On success, returns a pair of `(response_code, reply_length)`, giving the
/// server-generated response code and the number of bytes written into
/// `incoming` as a reply. `reply_length` is guaranteed by the kernel to be less
/// than or equal to `incoming.len()`, so it is safe to use it to slice
/// `incoming` without bounds checks by doing
/// `incoming.get_unchecked_mut(..reply_length)`.
///
/// # Errors
///
/// If `target` has the wrong generation number for the recipient, this returns
/// `Err(TaskDeath)`. This can also occur if the generation number is correct,
/// but the server crashes while preparing the response. There is currently no
/// way for you to distinguish these two cases, because nobody's asked for it.
///
/// # Panics
///
/// This call has no inherent panics, but the server being contacted is allowed
/// to force a panic in your task using [`sys_reply_fault`]. To avoid this, make
/// sure messages are valid, following whatever rules the server specifies in
/// its interface.
pub fn sys_send(
    target: TaskId,
    operation: u16,
    outgoing: &[u8],
    incoming: &mut [u8],
    leases: &mut [Lease<'_>],
) -> Result<(ResponseCode, usize), TaskDeath> {
    let _ = (target, operation, outgoing, incoming, leases);
    unimplemented!()
}

/// Convenience wrapper for [`sys_send`] for generating KIPCs.
///
/// Since the kernel cannot restart, this avoids the code related to the
/// `TaskDeath` error and simplifies callers.
///
/// As with `sys_send`, the kernel guarantees that the `usize` returned is less
/// than or equal to `incoming.len()`, making it safe to use it to slice
/// `incoming` without checks.
#[inline(always)]
pub fn sys_send_to_kernel(
    operation: u16,
    outgoing: &[u8],
    incoming: &mut [u8],
    leases: &mut [Lease<'_>],
) -> (ResponseCode, usize) {
    let target_and_operation = u32::from(TaskId::KERNEL.0) << 16 | u32::from(operation);
    let ret64 = unsafe {
        sys_send_stub(
            target_and_operation,
            outgoing.as_ptr(),
            outgoing.len(),
            incoming.as_mut_ptr(),
            incoming.len(),
            leases.as_mut_ptr().cast(),
            leases.len(),
        )
    };
    let retval = ResponseCode(ret64 as u32);
    (retval, (ret64 >> 32) as usize)
}

cfg_if::cfg_if! {
    if #[cfg(hubris_target = "riscv32imac-unknown-none-elf")] {
        global_asm!("
        .section .text.sys_send_stub
        .globl sys_send_stub
        .type sys_send_stub,function
        sys_send_stub:
            # Load in args from the struct.
            lw a6, 6*4(a0)
            lw a5, 5*4(a0)
            lw a4, 4*4(a0)
            lw a3, 3*4(a0)
            lw a2, 2*4(a0)
            lw a1, 1*4(a0)
            lw a0, 0*4(a0)

            # Load the constant syscall number.
            li a7, {sysnum}

            # To the kernel!
            ecall

            # Results are placed into the correct registers by the kernel, we can
            # just return now.
            ret
        ",
        sysnum = const Sysnum::Send as u32,
        );
    } else {
        compile_error!("unrecognized target for sys_send_stub");
    }
}

/// The actual return register layout after a call to `recv`.
///
/// This lets us blit the return registers directly into this struct from the
/// assembly stub without having to think too much.
#[repr(C)]
struct AbiRecvMessage {
    sender: u32,
    operation_or_notification: u32,
    sent_length: usize,
    reply_capacity: usize,
    lease_count: usize,
}

/// Receives a message from a waiting caller.
///
/// This finds a task waiting to send a message to the current task. The
/// caller's message is copied into `incoming`, and metadata about the message
/// is returned in a [`Message`] (wrapped in a [`MessageOrNotification`].
///
/// If `notification_mask` is not zero, and the current task has any pending
/// notifications, those notifications will be returned instead, using
/// [`MessageOrNotification::Notification`].
///
/// The `from` argument controls open vs closed receive. If `from` is `None`,
/// messages from any caller are accepted in priority order, so the returned
/// message will be the one from the highest-priority waiting task. If `from` is
/// `Some(t)`, only messages from `t` will be accepted. Notifications that pass
/// the `notification_mask` are always accepted. (If `t` is `TaskId::KERNEL`,
/// *only* notifications are accepted.)
///
/// # More convenient alternatives
///
/// This is the Swiss Army Chainsaw receive operation. It can do anything and is
/// difficult to hold correctly. Use `sys_recv` when you want to make decisions
/// about the behavior _dynamically:_
///
/// - Changing the notification mask depending on state.
/// - Switching between open and closed receive.
///
/// That's pretty rare in practice. If you don't need those features, consider
/// these alternatives, which are essentially convenience wrappers:
///
/// - [`sys_recv_msg`]: masks notifications. Unlike calling `sys_recv` with a
///   `notification_mask` of 0, `sys_recv_msg` doesn't make you write an
///   unreachable `Notification` case.
///
/// - [`sys_recv_open`]: accepts messages from any caller. This eliminates the
///   failure path that returns `TaskDeath`, saving you from needing to handle
///   it.
///
/// - [`sys_recv_msg_open`]: combines both of the above, for cases where you
///   don't want notifications and aren't picky about who's sending.
///
/// - [`sys_recv_notification`]: listens _only_ for notifications, leaving
///   messages queued. Very handy for waiting for interrupts only.
///
/// # Errors
///
/// If `from` is `Some(t)` and `t` has the wrong generation number --- or the
/// task described by `t` restarts while this task is blocked in `sys_recv` ---
/// this function returns `Err(TaskDeath)` to ensure that you don't wind up
/// waiting forever for a message that cannot arrive.
///
/// This call can only fail in closed receive mode, which is one of the reasons
/// why open receives are easier using [`sys_recv_open`] and kin.
pub fn sys_recv(
    incoming: &mut [MaybeUninit<u8>],
    notification_mask: u32,
    from: Option<TaskId>,
) -> Result<MessageOrNotification<'_>, TaskDeath> {
    let _ = (incoming, notification_mask, from);
    unimplemented!()
}

/// Convenience wrapper for `sys_recv` for the case where you know, statically,
/// that you don't care about notifications.
///
/// This is logically equivalent to `sys_recv(incoming, 0,
/// from)`, but without making you deal with the `Notification` case.
pub fn sys_recv_msg(
    incoming: &mut [MaybeUninit<u8>],
    from: Option<TaskId>,
) -> Result<Message<'_>, TaskDeath> {
    let _ = (incoming, from);
    unimplemented!()
}

/// Convenience wrapper for `sys_recv` for the case where you know, statically,
/// that any sender is fine.
///
/// This is logically equivalent to `sys_recv(incoming, notification_mask,
/// None)`, but without the code for handling errors in closed receive.
pub fn sys_recv_open(
    incoming: &mut [MaybeUninit<u8>],
    notification_mask: u32,
) -> MessageOrNotification<'_> {
    let _ = (incoming, notification_mask);
    unimplemented!()
}

/// Convenience wrapper for `sys_recv` for the case where you know, statically,
/// that any sender is fine and you're not interested in notifications.
///
/// This is logically equivalent to `sys_recv(incoming, 0, None)`, but without
/// the code for handling errors in closed receive, and without making you deal
/// with the `Notification` return case.
pub fn sys_recv_msg_open(incoming: &mut [MaybeUninit<u8>]) -> Message<'_> {
    let _ = incoming;
    unimplemented!()
}

/// Convenience wrapper for `sys_recv` for the case where code only cares about
/// notifications, and does _not_ want to receive IPCs.
///
/// This is logically equivalent to `sys_recv(&mut [], notification_mask,
/// Some(TaskId::KERNEL))`, but is implemented without any of the code related
/// to error checking or message handling.
#[inline(always)]
pub fn sys_recv_notification(notification_mask: u32) -> u32 {
    let mut out = MaybeUninit::<AbiRecvMessage>::uninit();
    let retval = unsafe {
        sys_recv_stub(
            core::ptr::null_mut(),
            0,
            notification_mask,
            0x8000_0000 | u32::from(TaskId::KERNEL.0),
            out.as_mut_ptr(),
        )
    };

    // This can't actually fail.
    let _ = retval;

    let rm = unsafe { out.assume_init() };
    rm.operation_or_notification
}

cfg_if::cfg_if! {
    if #[cfg(hubris_target = "riscv32imac-unknown-none-elf")] {
        global_asm!("
        .section .text.sys_recv_stub
        .globl sys_recv_stub
        .type sys_recv_stub,function
        sys_recv_stub:
            # Preserve output buffer pointer in callee-save register, ensuring
            # it is saved on the stack, which is kept 16-byte aligned.
            addi sp, sp, -4*4
            sw s2, 0*4(sp)
            mv s2, a4

            # Load the constant syscall number.
            li a7, {sysnum}

            # To the kernel!
            ecall

            # Write all the results out into the raw output buffer.
            sw a1, 0*4(s2)
            sw a2, 1*4(s2)
            sw a3, 2*4(s2)
            sw a4, 3*4(s2)
            sw a5, 4*4(s2)

            # Restore callee-save register and stack pointer and return.
            lw s2, 0*4(sp)
            addi sp, sp, 4*4
            ret
            ",
            sysnum = const Sysnum::Recv as u32,
        );
    } else {
        compile_error!("unrecognized target for sys_recv_stub");
    }
}

/// Replies to a previously received message, unblocking the sender.
///
/// This unblocks `sender` by delivering the response code `code`, and copying
/// the contents of `message` into the sender's provided incoming message
/// buffer.
///
/// Reply is a fire-and-forget operation in Hubris, so this doesn't return an
/// error code if `sender` has been forceably restarted after sending the
/// message. In practice, servers wind up ignoring that fact.
pub fn sys_reply(sender: TaskId, code: ResponseCode, message: &[u8]) {
    let _ = (sender, code, message);
    unimplemented!()
}

/// Replies to a previously received message by inducing a fault in the sender.
///
/// This delivers a synthetic (kernel-induced) fault in the task `sender`,
/// carrying `reason`. This operation is used to add protocol-specific
/// preconditions to IPC interactions. Common reasons to use reply-fault
/// include...
///
/// - Sender sent a message that is much too big or small for the protocol.
/// - Sender sent a bogus message that the server cannot decode.
/// - Sender did not provide enough space for a reply in the protocol.
/// - Sender included the wrong number of leases.
///
/// This also provides a way to implement protocol-specific access control.
/// Since the kernel always gives the server the sender's real `TaskId`, the
/// server can check it against a list of tasks permitted to perform any
/// given operation. If a task exceeds its permissions, the server can deliver
/// a fault with [`ReplyFaultReason::AccessViolation`].
pub fn sys_reply_fault(sender: TaskId, reason: ReplyFaultReason) {
    let _ = (sender, reason);
    unimplemented!()
}

/// Configures the calling task's kernel timer.
///
/// This sets the enable bit, deadline, and notification bitmask associated with
/// the task's timer. If `deadline` is `None`, the timer is disabled. If
/// `deadline` is `Some(t)`, the timer is enabled and set to go off at kernel
/// time `t`. (If `t` is the current time, or is in the past, the timer triggers
/// immediately.)
///
/// Either way, the timer is configured to post `bitmask` to the task's
/// notification bits when it triggers.
pub fn sys_set_timer(deadline: Option<u64>, bitmask: u32) {
    let _ = (deadline, bitmask);
    unimplemented!()
}

/// Reads the calling task's kernel timer state.
///
/// This returns a `TimerSettings` that includes the current timer enable
/// status, deadline, and notification bits, as well as the current kernel
/// timestamp.
///
/// This is the normal way of reading the kernel timestamp, even though it
/// returns a bit of extra information for that use case.
pub fn sys_get_timer() -> TimerSettings {
    unimplemented!()
}

/// Asks the kernel to enable a hardware interrupt mapped to this task.
///
/// The interrupt is identified using a notification mask, which must have
/// exactly one 1-bit. This gives a portable way for a task to talk about its
/// interrupts without needing to know the vendor-specific interrupt numbers. It
/// also means there's no way for a task to attempt to alter interrupts owned by
/// someone else, avoiding the need for access control checks.
///
/// # Panics
///
/// If `notification_mask` doesn't correspond to an actual hardware interrupt
/// mapped to the task, this triggers a fault. (It may be useful to loosen this
/// behavior to allow for simulated interrupts in test, but so far this has
/// always indicated a programming error.)
pub fn sys_enable_irq(notification_mask: u32) {
    let _ = notification_mask;
    unimplemented!()
}

/// Asks the kernel to enable a hardware interrupt mapped to this task, and
/// clear its pending status.
///
/// Performs the function of `sys_enable_irq`, but also clears the interrupt
/// controller's pending state for the relevant interrupt(s), if such a concept
/// exists on this architecture. If the architecture doesn't have a concept of
/// pending interrupts, this is equivalent to `sys_enable_irq`.
pub fn sys_enable_irq_and_clear_pending(notification_mask: u32) {
    let _ = notification_mask;
    unimplemented!()
}

/// Copies a chunk of data from borrowed memory into a local buffer.
///
/// This accesses memory borrowed through a lease provided by `lender`. The
/// lease is specified by its `index` in the lender's lease table, which will
/// normally be defined by the IPC protocol.
///
/// Data will be read starting `offset` bytes into the borrowed region, and
/// copied into `dest`. The number of bytes to read is given by `dest.len()`.
///
/// On success, returns the number of bytes read. In practice this can be lower
/// than `dest.len()` if it hits the end of the lender-provided space.
///
/// If the `lender` has been forceably restarted since the IPC began, or if
/// `lender` is not actually waiting in send (these two cases are identical from
/// the kernel's perspective), this returns `None` and the server should wrap up
/// processing the operation as if it were aborted.
///
/// # Panics
///
/// The server is expected to use the information provided in [`Message`] and
/// produced by [`sys_borrow_info`] to guide its access to leases. As a result,
/// attempting to access an invalid `index`, or attempting to read from a
/// write-only lease, are both treated as errors _in the server_ and trigger
/// faults.
pub fn sys_borrow_read(
    lender: TaskId,
    index: usize,
    offset: usize,
    dest: &mut [u8],
) -> Option<usize> {
    let _ = (lender, index, offset, dest);
    unimplemented!()
}

/// Copies a chunk of data from a local buffer into borrowed memory.
///
/// This accesses memory borrowed through a lease provided by `lender`. The
/// lease is specified by its `index` in the lender's lease table, which will
/// normally be defined by the IPC protocol.
///
/// Data will be written starting `offset` bytes into the borrowed region, and
/// sourced from `src`. The number of bytes to write is given by `src.len()`.
///
/// On success, returns the number of bytes written. In practice this can be
/// lower than `src.len()` if it hits the end of the lender-provided space.
///
/// If the `lender` has been forceably restarted since the IPC began, or if
/// `lender` is not actually waiting in send (these two cases are identical from
/// the kernel's perspective), this returns `None` and the server should wrap up
/// processing the operation as if it were aborted.
///
/// # Panics
///
/// The server is expected to use the information provided in [`Message`] and
/// produced by [`sys_borrow_info`] to guide its access to leases. As a result,
/// attempting to access an invalid `index`, or attempting to write to a
/// read-only lease, are both treated as errors _in the server_ and trigger
/// faults.
pub fn sys_borrow_write(lender: TaskId, index: usize, offset: usize, src: &[u8]) -> Option<usize> {
    let _ = (lender, index, offset, src);
    unimplemented!()
}

/// Collects information about borrowed memory available through a lease.
///
/// This returns a [`crate::BorrowInfo`] struct describing memory loaned by
/// `lender` through lease number `index`.
///
/// If `lender` has been forceably restarted since the IPC began, or if it isn't
/// actually waiting in an IPC (these conditions are equivalent to the kernel),
/// this returns `None`, and the server should treat the operation as aborted if
/// possible.
///
/// # Panics
///
/// The correct number of leases is provided to the server in [`Message`].
/// Attempting to access an `index` outside of that range is treated as an error
/// in the server, and induces a fault.
pub fn sys_borrow_info(lender: TaskId, index: usize) -> Option<crate::BorrowInfo> {
    let _ = (lender, index);
    unimplemented!()
}

/// Posts some number of notifications to a task.
///
/// This call merges `notifications` into the set of pending notifications for
/// `task` by bitwise-OR. It is legal, though wasteful, for `notifications` to
/// be zero.
///
/// If the generation number in `task` is wrong, returns `Err(TaskDeath)`.
pub fn sys_post(task: TaskId, notifications: u32) -> Result<(), TaskDeath> {
    let _ = (task, notifications);
    unimplemented!()
}

extern "C" {
    /// Low-level send syscall stub.
    ///
    /// # Safety
    ///
    /// To use this safely, all of the (base,len) pointers must meet the
    /// validity rules for slice references. The easiest way to ensure this is
    /// to derive them directly from slice references.
    ///
    /// This also implies that the outgoing, incoming, and lease regions may not
    /// overlap.
    ///
    /// As an optimization, the memory pointed to by `incoming_base` need not be
    /// initialized, and so it is safe to have derived the `incoming_base`
    /// pointer from an array of `MaybeUninit<u8>`. Once this returns, you can
    /// assume that the _prefix_ of the `incoming` slice up to the response
    /// length has been initialized. The tail of that buffer may _not_ have been
    /// initialized.
    fn sys_send_stub(
        target_and_operation: u32,
        outgoing_base: *const u8,
        outgoing_len: usize,
        incoming_base: *mut u8,
        incoming_len: usize,
        lease_base: *const AbiLease,
        lease_count: usize,
    ) -> u64;

    /// Low-level recv syscall stub.
    ///
    /// # Safety
    ///
    /// To use this safely, the incoming base/len pointers must meet the
    /// validity rules for a slice reference. The easiest way to ensure this is
    /// to derive them directly from a slice reference.
    ///
    /// This also implies that the `incoming` slice and `out` pointee may not
    /// overlap.
    ///
    /// As an optimization, the memory pointed to by `out` need not be
    /// initialized, and so it is safe to have derived the `out` pointer from a
    /// `MaybeUninit<AbiRecvMessage>`. Once this returns, you can assume the
    /// memory has been initialized.
    fn sys_recv_stub(
        incoming_base: *mut u8,
        incoming_len: usize,
        notification_mask: u32,
        sender_bits: u32,
        out: *mut AbiRecvMessage,
    ) -> u32;
}

cfg_if::cfg_if! {
    if #[cfg(hubris_target = "riscv32imac-unknown-none-elf")] {
        // Add startup code for RISC-V
        global_asm!("
        .section .text.start, \"ax\"
        .global _start
        _start:
            # Copy .data from flash to RAM
            la t0, __sdata
            la a3, __edata
            la t1, __sidata
            bgeu t0, a3, 2f
        1:  
            lw t2, 0(t1)
            addi t1, t1, 4
            sw t2, 0(t0)
            addi t0, t0, 4
            bltu t0, a3, 1b
        2:  
            # Zero out .bss
            la t0, __sbss
            la t2, __ebss
            bgeu  t0, t2, 4f
        3:  
            sw  zero, 0(t0)
            addi t0, t0, 4
            bltu t0, t2, 3b
        4: 
            # Now, to the user entry point. We call it in case it
            # returns. (It's not supposed to.) We reference it through
            # a sym operand because it's a Rust func and may be mangled.
            jal ra, {main}

            # Should main return... kill it.
            unimp
            ",
            main = sym main,
        );
    } else {
        compile_error!("unrecognized target for start code");
    }
}
