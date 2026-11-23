//! Kernel Logging and Console Output
//! Mayaank N - Micro-OS (25BTRCL148)

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        $crate::drivers::_serial_print(format_args!($($arg)*));
        $crate::drivers::_vga_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! kprintln {
    () => ($crate::kprint!("\n"));
    ($($arg:tt)*) => {
        $crate::kprint!("{}\n", format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! klog_info {
    ($($arg:tt)*) => {
        $crate::kprintln!("[INFO ] [{:>6} ticks] {}", $crate::timer::current_ticks(), format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! klog_warn {
    ($($arg:tt)*) => {
        $crate::kprintln!("[WARN ] [{:>6} ticks] {}", $crate::timer::current_ticks(), format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! klog_trace {
    ($($arg:tt)*) => {
        $crate::kprintln!("[TRACE] [{:>6} ticks] {}", $crate::timer::current_ticks(), format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! klog_sched {
    ($($arg:tt)*) => {
        $crate::kprintln!("[SCHED] [{:>6} ticks] {}", $crate::timer::current_ticks(), format_args!($($arg)*));
    };
}
