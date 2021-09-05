extern "C" {
    // 获取当前的线程
    pub fn rt_thread_self() -> *mut usize;

    // 挂起当前的线程 0: 成功 -1:当前线程未运行
    pub fn rt_thread_suspend(th: *mut usize) -> isize;

    pub fn rt_schedule();

    // 恢复当前的线程 0:成功 -1：当前线程未挂起
    pub fn rt_thread_resume(th: *mut usize) -> isize;

    // 关闭中断
    pub fn rt_hw_interrupt_disable() -> usize;
    // 打开中断
    pub fn rt_hw_interrupt_enable(val: usize);
}

pub fn no_irq<F, T>(f: F) -> T
where
    F: FnOnce() -> T,
{
    let level;
    let out;
    unsafe {
        level = rt_hw_interrupt_disable();
    }
    out = f();
    unsafe {
        rt_hw_interrupt_enable(level);
    }
    out
}
