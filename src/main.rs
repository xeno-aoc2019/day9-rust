extern crate num_bigint;
extern crate num_traits;

use std::fmt;
use std::fs::File;
use std::path::Path;
use std::io::{BufRead, BufReader, Result, Lines};
use std::fmt::Formatter;
use num_bigint::BigInt;
use num_traits::{Zero, One, ToPrimitive};
use std::ops::Add;
use std::collections::HashMap;


struct Instruction {
    opcode: i32,
    steps_next: usize,
}

const I_ADD: Instruction = Instruction { opcode: 1, steps_next: 4 };
const I_MUL: Instruction = Instruction { opcode: 2, steps_next: 4 };
const I_IN: Instruction = Instruction { opcode: 3, steps_next: 2 };
const I_OUT: Instruction = Instruction { opcode: 4, steps_next: 2 };
const I_JT: Instruction = Instruction { opcode: 5, steps_next: 3 };
const I_JF: Instruction = Instruction { opcode: 6, steps_next: 3 };
const I_LT: Instruction = Instruction { opcode: 7, steps_next: 4 };
const I_EQ: Instruction = Instruction { opcode: 8, steps_next: 4 };
const I_RBO: Instruction = Instruction { opcode: 9, steps_next: 2 };
const I_HALT: Instruction = Instruction { opcode: 99, steps_next: 0 };

const MODE_REF: i32 = 0;
const MODE_VAL: i32 = 1;
const MODE_REL: i32 = 2;

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.opcode {
            1 => write!(f, "I_ADD({})", self.opcode),
            2 => write!(f, "I_MUL({})", self.opcode),
            3 => write!(f, "I_IN({})", self.opcode),
            4 => write!(f, "I_OUT({})", self.opcode),
            5 => write!(f, "I_JT({})", self.opcode),
            6 => write!(f, "I_JF({})", self.opcode),
            7 => write!(f, "I_LT({})", self.opcode),
            8 => write!(f, "I_EQ({})", self.opcode),
            9 => write!(f, "I_RBO({}", self.opcode),
            _ => write!(f, "UNKNOWN({}", self.opcode)
        }
    }
}

struct Param {
    value: i32,
    mode: i32,
}

impl Param {
    fn new(value: i32, mode: i32) -> Param {
        Param {
            value,
            mode,
        }
    }

    fn is_valid(&self) -> bool {
        if !(self.mode == 0 || self.mode == 1) { return false; }
        if self.mode == 0 && self.value < 0 { return false; }
        true
    }

    fn is_reference(&self) -> bool {
        return self.mode == MODE_REF;
    }

    fn is_value(&self) -> bool {
        return self.mode == MODE_VAL;
    }
}

struct ParaModes {
    modes: [i32; 3]
}

impl ParaModes {
    fn param_modes(instr: i32) -> [i32; 3] {
        let mut params: [i32; 3] = [0; 3];
        let param_part = (instr - instr % 100) / 100;
        params[0] = param_part % 10;
        params[1] = ((param_part - param_part % 10) / 10) % 10;
        params[2] = ((param_part - (param_part % 100)) / 100) % 10;
//        println!("MODES: instr={} : {} => {},{},{}", instr, param_part, params[0], params[1], params[2]);
        params
    }

    fn new(instr: i32) -> ParaModes {
        ParaModes {
            modes: ParaModes::param_modes(instr)
        }
    }
    fn mode(&self, n: i32) -> i32 {
        match n {
            1 => self.modes[0],
            2 => self.modes[1],
            3 => self.modes[2],
            _ => panic!("Unsupported parameter mode number")
        }
    }
}

impl fmt::Display for ParaModes {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Modes({} {} {})", self.modes[0], self.modes[1], self.modes[2])
    }
}

impl fmt::Display for VM {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "VM(ip={} input=", self.ip);
        let mut inp_ind = 0;
        for value in self.inputs.iter() {
            if inp_ind == self.in_p {
                write!(f, "[{}] ", value);
            } else {
                write!(f, "{} ", value);
            }
            inp_ind += 1;
        }
        write!(f, "[");
        if self.halted {
            write!(f, "H");
        }
        if self.interrupted {
            write!(f, "I");
        }
        write!(f, "]");
        write!(f, " output=");
        for value in self.outputs.iter() {
            write!(f, "{} ", value);
        }
        write!(f, " program=");
        for value in self.program.iter() {
            write!(f, "{} ", value);
        }
        write!(f, ")")
    }
}

struct VM {
    program: Vec<BigInt>,
    ip: BigInt,
    rb: BigInt,
    in_p: i32,
    out_p: i32,
    out_rp: i32,
    halted: bool,
    interrupted: bool,
    inputs: Vec<BigInt>,
    outputs: Vec<BigInt>,
    high_mem: HashMap<BigInt, BigInt>,
}

impl VM {
    fn new(program: Vec<BigInt>, inputs: Vec<BigInt>) -> VM {
        VM {
            program,
            ip: Zero::zero(),
            rb: Zero::zero(),
            in_p: 0,
            out_p: 0,
            out_rp: 0,
            halted: false,
            interrupted: false,
            inputs,
            outputs: vec!(),
            high_mem: HashMap::new(),
        }
    }

    fn read_mem(&self, addr: BigInt) -> BigInt {
        if addr < Zero::zero() {
            println!("Tried to read a negative memory address: {}", addr);
            panic!("Illegal memory access");
        }
        if addr > BigInt::from(self.program.len()) {
            let val = match self.high_mem.get(&addr) {
                Some(value) => value.clone(),
                None => Zero::zero()
            };
            println!("read_mem: high_mem: {} -> {}", addr, val);
            return val;
        }
        let value = self.program[addr.to_u32().unwrap() as usize].clone();
        println!("Reading [{}] = {}", addr, value);
        value
    }

    fn write_mem(&mut self, addr: BigInt, value: BigInt) {
        if addr < Zero::zero() {
            println!("Tried to write to a negative memory address: {}", addr);
            panic!("Illegal memory access");
        }
        if (addr > BigInt::from(self.program.len())) {
            // println!("write_mem: high_mem: {}", addr.clone());
            self.high_mem.insert(addr, value);
        } else {
            // println!("Writing [{}] = {}", addr, value);
            self.program[addr.to_u32().unwrap() as usize] = value;
        }
    }


    fn fetch_instr(&self) -> (Instruction, ParaModes) {
        let instruction = self.program[self.ip.to_u32().unwrap() as usize].clone().to_u32().unwrap();
        let para_modes = ParaModes::new(instruction.to_i32().unwrap());
//        println!("Fetching instruction at [{}] = {}", self.ip, instruction);
        let opcode = instruction % 100;
        let instr = match opcode {
            1 => I_ADD,
            2 => I_MUL,
            3 => I_IN,
            4 => I_OUT,
            5 => I_JT,
            6 => I_JF,
            7 => I_LT,
            8 => I_EQ,
            9 => I_RBO,
            99 => I_HALT,
            _ => {
                println!("Unknown opcode at ip={}: {}", self.ip, opcode);
                panic!("Uknown opcode")
            }
        };
        (instr, para_modes)
    }

    fn fetch_arg(&self, n: BigInt) -> BigInt {
        self.program[self.ip.clone().add(n).to_usize().unwrap()].clone()
    }

    fn fetch_arg_value(&self, n: BigInt, mode: i32) -> BigInt {
        let arg = self.program[(self.ip.clone() + n.clone()).to_usize().unwrap()].clone();
//        println!("fetch_arg_value({}, {})", n.clone(), mode);
        if mode == MODE_VAL {
            return arg;
        }
        if mode == MODE_REF {
            return self.read_mem(arg);
        }
        if mode == MODE_REL {
            let index = arg + self.rb.clone();
            return self.read_mem(index);
        }
        panic!("Unknown param mode");
    }

    fn step(&mut self, n: usize) {
        self.ip += n;
    }

    fn goto(&mut self, dest: BigInt) {
        println!("Goto {}", dest);
        if dest < Zero::zero() {
            panic!("Trying to jump out of the program");
        }
        self.ip = dest;
    }

    fn read_input(&mut self) -> Option<BigInt> {
        if self.has_input() {
            let input = self.inputs[self.in_p as usize].clone();
            self.in_p += 1;
            Some(input)
        } else {
            None
        }
    }

    fn i_add(&mut self, modes: &ParaModes) {
        let param1 = self.fetch_arg_value(One::one(), modes.mode(1));
        let param2 = self.fetch_arg_value(BigInt::from(2), modes.mode(2));
        let dest = self.fetch_arg(BigInt::from(3));
        println!("I_ADD [{}] = {}+{}", dest, param1, param2);
        self.write_mem(dest, param1 + param2);
        self.step(I_ADD.steps_next);
    }

    fn i_mul(&mut self, modes: &ParaModes) {
        let adr1 = self.fetch_arg(One::one());
        let adr2 = self.fetch_arg(BigInt::from(2));
        let param1 = self.fetch_arg_value(One::one(), modes.mode(1));
        let param2 = self.fetch_arg_value(BigInt::from(2), modes.mode(2));
        let dest = self.fetch_arg(BigInt::from(3));
//        println!("I_MUL [{}] = [{}]+[{}]", dest, adr1, adr2);
        println!("I_MUL [{}] = [{}]={}*[{}]={}", dest, adr1, param1, adr2, param2);
        let value = param1 * param2;
        self.write_mem(dest, value);
        self.step(I_MUL.steps_next);
    }

    fn has_input(&self) -> bool {
        self.inputs.len() > (self.in_p as usize)
    }

    fn i_input(&mut self) {
        self.has_input();
        let adr = self.fetch_arg(One::one());
        let input = self.read_input();
        match input {
            Some(input) => {
                self.write_mem(adr.clone(), input.clone());
                println!("I_INPUT [{}] input:{}", adr.clone(), input.clone());
                self.ip = self.ip.clone() + I_IN.steps_next;
            }
            None => {
                println!("Interupting, awaiting IO");
                self.interrupted = true;
            }
        }
    }

    fn i_output(&mut self, modes: &ParaModes) {
        let output = self.fetch_arg_value(One::one(), modes.mode(1));
        self.outputs.push(output.clone());
        self.out_p += 1;
        println!("I_OUTPUT: outputting {}", output.clone());
        self.ip = self.ip.clone() + I_OUT.steps_next;
    }

    fn i_rbo(&mut self, modes: &ParaModes) {
        let value = self.fetch_arg_value(One::one(), modes.mode(1));
        println!("RBO: rb={} += {}", self.rb, value);
        self.rb += value;
        self.ip = self.ip.clone() + I_RBO.steps_next;
    }

    fn i_jt(&mut self, modes: &ParaModes) {
        let param = self.fetch_arg_value(One::one(), modes.mode(1));
        let dest = self.fetch_arg_value(BigInt::from(2), modes.mode(2));
        let jump = param != Zero::zero();
        println!("I_JT {} ->{}:{}", dest, dest, jump);
        if jump {
            self.goto(dest);
        } else {
            self.step(I_JT.steps_next);
        }
    }

    fn i_jf(&mut self, modes: &ParaModes) {
        let param = self.fetch_arg_value(One::one(), modes.mode(1));
        let dest = self.fetch_arg_value(BigInt::from(2), modes.mode(2));
        let jump = param == Zero::zero();
        println!("I_JF {} ->{}:{}", param, dest, jump);
        if jump {
            self.goto(dest);
        } else {
            self.step(I_JT.steps_next);
        }
    }

    fn i_lt(&mut self, modes: &ParaModes) {
        let param1 = self.fetch_arg_value(One::one(), modes.mode(1));
        let param2 = self.fetch_arg_value(BigInt::from(2), modes.mode(2));
        let dest = self.fetch_arg(BigInt::from(3));
        let res = if param1 < param2 { One::one() } else { Zero::zero() };
        println!("I_LT [{}] := {} ( {} < {} )", dest, res, param1, param2);
        self.write_mem(dest, res);
        self.step(I_LT.steps_next);
    }

    fn i_eq(&mut self, modes: &ParaModes) {
        let param1 = self.fetch_arg_value(One::one(), modes.mode(1));
        let param2 = self.fetch_arg_value(BigInt::from(2), modes.mode(2));
        let dest = self.fetch_arg(BigInt::from(3));
        let res = if param1 == param2 { One::one() } else { Zero::zero() };
        println!("I_EQ [{}] := {} ( {}=={} )", dest, res, param1, param2);
        self.write_mem(dest, res);
        self.step(I_EQ.steps_next);
    }

    fn i_halt(&mut self) {
        println!("I_HALT");
        self.halted = true;
    }

    fn add_input(&mut self, input: BigInt) {
        self.inputs.push(input);
        self.interrupted = false;
    }

    fn read_output(&mut self) -> BigInt {
        println!("READ OUTPUT: {} {}", self.outputs.len(), self.out_rp);
        let outv = self.outputs[self.out_rp as usize].clone();
        self.out_rp += 1;
        outv
    }

    fn exec_inst(&mut self) {
        let (instr, modes) = self.fetch_instr();
        let opcode = instr.opcode;
        println!("Executing: {} ip={} {}", opcode, self.ip, modes);
        if opcode == 99 { return self.i_halt(); };
        if opcode == 1 { return self.i_add(&modes); };
        if opcode == 2 { return self.i_mul(&modes); };
        if opcode == 3 { return self.i_input(); };
        if opcode == 4 { return self.i_output(&modes); };
        if opcode == 5 { return self.i_jt(&modes); };
        if opcode == 6 { return self.i_jf(&modes); };
        if opcode == 7 { return self.i_lt(&modes); };
        if opcode == 8 { return self.i_eq(&modes); };
        if opcode == 9 { return self.i_rbo(&modes); };
        println!("Unknown instruction: {}, halting", opcode);
        self.i_halt();
    }

    fn is_runnable(&self) -> bool {
        !self.halted && !self.interrupted
    }

    fn resume(&mut self) {
        println!("resuming vm={}", self);
        self.interrupted = false;
        while self.is_runnable() {
            self.exec_inst();
        }
        println!("end vm={}", self);
    }

    fn run(&mut self) {
        println!("start vm={}", self);
        self.ip = Zero::zero();
        while self.is_runnable() {
            self.exec_inst();
        }
        println!("end vm={}", self);
    }
}

fn main() {
    let program = read_program();
    // let program = vec!(3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0);
    // let program = vec!(3,0,4,0,99);

    let mut vm = VM::new(program.clone(), vec!());
    vm.run();
    vm.add_input(One::one());
    println!("{}", vm);
    vm.resume();
    vm.add_input(BigInt::from(4));
    vm.resume();
    println!("Output: {}", vm.read_output())
}

fn read_program() -> Vec<BigInt> {
    if let Ok(lines) = get_lines("input.txt") {
        for maybe_line in lines {
            if let Ok(line) = maybe_line {
                let mut result: Vec<BigInt> = vec!();
                for item in line.split(",") {
                    let byte: BigInt = item.parse::<BigInt>().unwrap();
                    result.push(byte);
                }
                return result;
            }
        }
    }
    panic!("no input");
}

fn get_lines<P>(file_name: P) -> Result<Lines<BufReader<File>>>
    where P: AsRef<Path>, {
    let file = File::open(file_name)?;
    Ok(BufReader::new(file).lines())
}