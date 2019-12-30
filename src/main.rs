mod input;
mod layer;

use layer::Layer;

fn main() {
    let layers = load_layers();
    println!("Loaded {} layers ", layers.len());
    let mut leastZeros = 151;
    let mut value = 0;
    for layer in &layers {
        let zeroCount = layer.digit_count(0);
        if zeroCount <= leastZeros {
            leastZeros = zeroCount;
            value = layer.digit_count(1) * layer.digit_count(2);
        }
    }
    for y in 0..6 {
        for x in 0..25 {
            let pixel = find_pixel(&layers, x, y);
            if pixel == 0 {
                print!(" ");
            }
            if pixel == 1 {
                print!("#");
            }
            if pixel == 2 {
                print!(" ");
            }
        }
        println!();
    }

    println!("Task 1: {}", value);
}

fn load_layers() -> Vec<Layer> {
    let mut layers: Vec<Layer> = vec!();
    let digits = input::read_input_digits();
    let layerCount = digits.len() / 150;
    if layerCount * 150 != digits.len() {
        println!("Size mismatch: {} not divisable by 150", digits.len());
    }
    for layerNumber in 0..layerCount {
        let layer = Layer::new(&digits, layerNumber * 150);
        layers.push(layer);
    }
    layers
}

fn find_pixel(layers: &Vec<Layer>, x: u32, y: u32) -> u32 {
    for layer in layers {
        let pixel = layer.getPixel(x, y);
        if pixel != 2 {
            return pixel;
        }
    }
    return 2;
}