static mut i: i32 = 0;

#[no_mangle]
pub extern "C" fn update() -> i32 {
    return unsafe {
        i += 1;
        i
    }
} 

pub fn main() {
    println!("Hello, World!"); 
}
