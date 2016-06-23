// This is a comment, and will be ignored by the compiler
// You can test this code by clicking the "Run" button over there ->
// or if prefer to use your keyboard, you can use the "Ctrl + Enter" shortcut

// This code is editable, feel free to hack it!
// You can always return to the original code by clicking the "Reset" button ->

// This is the main function
fn main() {
    // The statements here will be executed when the compiled binary is called

    // Print text to the console
    println!("Hello World!");
    let test: u8 = 0x34;
    let test2: u8=0x21;
    println!("got now {:?}", ( (test as i16) + ((test2 as u16) << 8) as i16));

}
