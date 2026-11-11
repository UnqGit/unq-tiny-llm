use std::io::Write;

#[allow(unused)]
pub struct Fax {
    load_bar_len: usize,
    printed_load_bar: bool,
    loading_bar_activated: bool,
}

#[allow(unused)]
impl Fax {
    pub fn new() -> Self {
        Fax { load_bar_len: 0, printed_load_bar: false, loading_bar_activated: false }
    }

    fn clear_loading_bar(&mut self) {
        if self.printed_load_bar {
            print!("\r\x1b[2K");
        }
        self.printed_load_bar = false;
    }

    pub fn loading_bar_set(&mut self, size: usize) {
        assert!(size != 0);
        self.clear_loading_bar();
        self.load_bar_len = size;
        self.loading_bar_activated = true;
    }
    
    pub fn loading_bar_del(&mut self) {
        self.clear_loading_bar();
        self.load_bar_len = 0;
        self.loading_bar_activated = false;
    }

    pub fn loading_bar(&mut self, progress: f32) {
        assert!(self.loading_bar_activated);
        assert!(progress >= 0.0 && progress <= 1.0);

        let fill = (self.load_bar_len as f32 * progress) as usize;
        let clear = self.load_bar_len - fill;

        self.clear_loading_bar();
        print!("\r[{}{}]{:6.2}%", "=".repeat(fill), " ".repeat(clear), progress * 100.0);
        self.printed_load_bar = true;
        std::io::stdout().flush().unwrap();
    }
}

pub const RED: u8 = 31;
pub const GREEN: u8 = 32;
pub const YELLOW: u8 = 33;
pub const WHITE: u8 = 37;

#[macro_export]
macro_rules! print_colored {
    ($color:expr, $($arg:tt)*) => {
        println!(
            "\x1b[{}m{}\x1b[0m",
            $color,
            format_args!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log {
    ($name:literal, $color:expr, $($arg:tt)*) => {
        println!(
            "\x1b[{}m[{}]: \x1b[0m{}",
            $color,
            $name,
            format_args!($($arg)*)
        )
    };
}

#[macro_export]
macro_rules! log_success {
    ($($arg:tt)*) => {
        $crate::log!("SUCCESS", $crate::logger::GREEN, $($arg)*);
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::log!("INFO", $crate::logger::WHITE, $($arg)*)
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::log!("ERROR", $crate::logger::RED, $($arg)*)
    };
}

#[macro_export]
macro_rules! log_warning {
    ($($arg:tt)*) => {
        $crate::log!("WARNING", $crate::logger::YELLOW, $($arg)*)
    };
}