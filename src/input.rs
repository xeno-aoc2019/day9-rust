use std::fs;

pub fn read_input_digits() -> Vec<u32> {
    let mut digits: Vec<u32> = vec!();
    let content = fs::read_to_string("input.txt")
        .expect("Failed to read the file");
    println!("{}", content);
    for character in content.chars() {
        let c = character.to_digit(10);
        if let Some(digit) = c {
            digits.push(digit);
        }
    }
    return digits;
}
