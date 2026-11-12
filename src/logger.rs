use std::io::Write;

pub struct LoadingBar {
    load_bar_len: usize,
    target_count: usize,
    current_count: usize,
}

impl LoadingBar {
    pub fn create(target: usize, len: usize) -> Self {
        LoadingBar { load_bar_len: len, current_count: 0, target_count: target }
    }

    pub fn clear(&self) {
        print!("\r\x1b[2K");
    }

    pub fn show(&mut self, current_count: usize) {
        assert!(current_count <= self.target_count);

        let progress = current_count as f32 / self.target_count as f32;
        let fill = (self.load_bar_len as f32 * progress) as usize;
        let clear = self.load_bar_len - fill;
        self.current_count = current_count;
        
        self.clear();
        print!("\r[{}{}] {}/{} {:6.2}%", "=".repeat(fill), " ".repeat(clear), current_count, self.target_count, progress * 100.0);
        std::io::stdout().flush().unwrap();
    }
}

pub const RED: u8 = 31;
pub const GREEN: u8 = 32;
pub const YELLOW: u8 = 33;
pub const CYAN: u8 = 36;
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