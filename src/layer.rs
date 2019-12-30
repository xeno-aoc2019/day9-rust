pub struct Layer {
    pixels: [u32; 150]
}

fn toIndex(x: u32, y: u32) -> usize {
    return (y * 25 + x) as usize;
}


impl Layer {
    pub fn getPixel(&self, x: u32, y: u32) -> u32 {
        let index = toIndex(x, y);
        return self.pixels[index];
    }

    pub fn setPixel(&mut self, x: u32, y: u32, value: u32) {
        let index = toIndex(x, y);
        self.pixels[index] = value;
    }

    pub fn new(values: &Vec<u32>, startIndex: usize) -> Layer {
        let mut l: Layer = Layer { pixels: [0; 150] };
        for i in 0..150 {
            l.pixels[i] = values[startIndex + i];
            // println!("{}", i);
        }
        return l;
    }
    pub fn digit_count(&self, digit: u32) -> u32 {
        let mut count = 0;
        for i in 0..150 {
            if self.pixels[i] == digit {
                count = count + 1;
            }
        }
        count
    }
}