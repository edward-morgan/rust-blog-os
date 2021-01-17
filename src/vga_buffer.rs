use lazy_static::lazy_static;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
  Black = 0,
  Blue = 1,
  Green = 2,
  Cyan = 3,
  Red = 4,
  Magenta = 5,
  Brown = 6,
  LightGray = 7,
  DarkGray = 8,
  LightBlue = 9,
  LightGreen = 10,
  LightCyan = 11,
  LightRed = 12,
  Pink = 13,
  Yellow = 14,
  White = 15
}
// PartialEq => symmetry and transitivity
// Debug => format via {:?}
// repr(u8) => represent each enum as a u8

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);
// Implementing a newtype allows us to type-check while still
// using u8 data types!
impl ColorCode {
  fn new(foreground: Color, background: Color) -> ColorCode {
    ColorCode((background as u8) << 4 | (foreground as u8))
  }
}

// repr(C) interprets structs as C structs ( ex. fixed struct 
// field ordering, which is not available in Rust structs)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
  ascii_char: u8,
  color_code: ColorCode,
}

// Size of the VGA buffer
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// We have to make Buffer::chars Volatile so it won't get optimized away; since we're just writing to it and never
// reading from it, the compiler might decide not to include writes to it (not knowing about the screen printing)
use volatile::Volatile;
#[repr(transparent)]
struct Buffer {
  chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// Specify 'static to buffer because the buffer will always be around
pub struct Writer {
  column_position: usize,
  color_code: ColorCode,
  buffer: &'static mut Buffer,
}

impl Writer {
  pub fn write_byte(&mut self, byte: u8) {
    match byte {
      b'\n' => self.new_line(),
      byte => {
        if self.column_position >= BUFFER_WIDTH {
          self.new_line();
        }

        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;

        let color_code = self.color_code;
        self.buffer.chars[row][col].write(ScreenChar {
          ascii_char: byte,
          color_code: color_code,
        });
        self.column_position += 1;
      }
    }
  }

  fn new_line(&mut self) { 
   for row in 1..BUFFER_HEIGHT {
     for col in 0..BUFFER_WIDTH {
       // Use read() and write() because each value is wrapped in Volatile
      let c = self.buffer.chars[row][col].read();
      self.buffer.chars[row-1][col].write(c);
     }
   }
   self.clear_row(BUFFER_HEIGHT - 1);
   self.column_position = 0;
  }

  fn clear_row(&mut self, row: usize) {
    let blank = ScreenChar {
      ascii_char: b' ',
      color_code: self.color_code
    };
    for col in 0..BUFFER_WIDTH {
      self.buffer.chars[row][col].write(blank);
    }
  }

  pub fn write_string(&mut self, s: &str) {
    for byte in s.bytes() {
      match byte {
      0x20..=0x7e | b'\n' => self.write_byte(byte),
      _ => self.write_byte(0xfe),
      }
    }
  }
}

// Implement the Write trait for Writer (only one reqd method)
// So we can write integers/floats easily using core::fmt::Write
use core::fmt;
impl fmt::Write for Writer {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    self.write_string(s);
    Ok(())
  }
}

/* We want to use lazy statics because Writer will use unsafe raw pointers, which cannot be evaluated at compile-time
 * (which is when Rust computes statics). Using lazy_static will initialize the static when it is first used.
 */
//use lazy_static::lazy_static;
/* In order to use the Writer outside of this class, we need to make it mutable yet global, which is difficult to do.
 * The best way to do this without using Rust constructs we don't have access to is to wrap it in a Mutex, thus allowing
 * safe "interior mutability."
*/
use spin::Mutex;
lazy_static! {
  pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
    column_position: 0,
    color_code: ColorCode::new(Color::Yellow, Color::Black),
    buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
  });
}

///// Macros for printing

#[macro_export]
macro_rules! print {
  /* Pattern matching in Rust macros is a DSL w/separate grammar.   
   * - essentially pattern matching on Rust code
   * () around each arm of the match
   * $() captures values that match the pattern inside
   * $arg:tt indicates an argument of type tt (TOKEN)  
   * * is a repetition value (either *, + or ? (only in Rust 2018+))
   */
  ($($arg:tt)*) => (
    // The `crate` keyword allows the macro to be used both inside this file and outside
    // It expands to the current crate (vga_buffer) if used outside the file
    $crate::vga_buffer::_print(format_args!($($arg)*))
  );
}

#[macro_export]
macro_rules! println {
  () => ( $crate::print!("\n") );
  ($($arg:tt)*) => (
    $crate::print!("{}\n", format_args!($($arg)*))
  );
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
  use core::fmt::Write;
  WRITER.lock().write_fmt(args).unwrap()
}
