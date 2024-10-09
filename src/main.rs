use std::env;
use std::fs;
use std::thread;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::WindowCanvas;
use rand::Rng;


struct Registers {
    V: [u8; 16],
    DT: u8,
    ST: u8,
    I: u16,
    SP: u16,
    PC: u16,
}

impl Default for Registers {
    fn default() -> Registers {
        Registers { 
            I: 0,
            V: [0; 16],
            DT: 0,
            ST: 0,
            SP: 0xfa0,
            PC: 0x200
        }
    }
}

const chip8_sprites: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

fn get_bit(byte: u8, pos: u8) -> u8 {
    if(pos == 8) {
        return byte & 0x01;
    } 
    
   return ((byte >> (8-pos)) & 0x1) as u8;
}

fn update_display_mem(display_mem: &mut [[u8; 64]; 128], reg: &mut Registers, X: usize, 
    Y: usize, memory: &mut [u8; 4096], N: usize) {

    let mut x_pos = reg.V[X] as usize;
    let mut y_pos = reg.V[Y] as usize;
    let index = reg.I as usize; 

    for byte in &memory[index..(index+N)] {
        for i in 1..=8 {
            //println!("xy value {:x} xor value {:x}",display_mem[x_pos][y_pos], (display_mem[x_pos][y_pos] as u8) ^ (get_bit(*byte, i) as u8));
            if (display_mem[x_pos][y_pos] == 0x1) && ((display_mem[x_pos][y_pos] as u8) ^ (get_bit(*byte, i) as u8) == 0x0)  {
                reg.V[0xF] = 1;
            } else {
                reg.V[0xF] = 0;
            }
            //println!("display VF value {:x}", reg.V[0xF]);

            display_mem[x_pos][y_pos] = (display_mem[x_pos][y_pos] as u8) ^ (get_bit(*byte, i) as u8);
            //println!("display xor result {:x}", display_mem[x_pos][y_pos]);
            x_pos = x_pos+1;
        }
        y_pos = y_pos+1;
        x_pos = reg.V[X] as usize;
    }
}

fn display(canvas: &mut WindowCanvas, reg: &mut Registers, X: usize, 
    Y: usize, memory: &mut [u8; 4096], N: usize) {
   
   let mut x_pos = (reg.V[X] as i32)*8;
   let mut y_pos = (reg.V[Y] as i32)*8;
   let index = reg.I as usize; 
   //println!("display at X: {} and Y: {} this sprite info: {:x?}", reg.V[X], reg.V[Y],
   //     &memory[index..(index+N)]);
    
    for byte in &memory[index..(index+N)] {
        for i in 1..=8 {
            if get_bit(*byte, i) == 1 {
                canvas.set_draw_color(Color::RGB(255,255,25));
                canvas.fill_rect(Rect::new(x_pos, y_pos, 8, 8));
                canvas.present();
            } else {
                canvas.set_draw_color(Color::RGB(0,0,255));
                canvas.fill_rect(Rect::new(x_pos, y_pos, 8, 8));
                canvas.present();
            }
            thread::sleep_ms(2);
            x_pos = x_pos + 8;
        }
        x_pos = (reg.V[X] as i32)*8;
        //x_pos = x_pos*8;
        y_pos = y_pos + 8;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let chp8_file = &args[1];
    
    let chp8_file_contents = fs::read(chp8_file)
        .expect("Something went wrong with reading the file");

    println!("{:#?}", args);
    
    chp8_dissassemble(&chp8_file_contents);


}

fn init_memory(chp8_code: &Vec<u8>, memory: &mut [u8; 4096]) {
    let mut pos = 0;
    for i in &mut memory[0x0..0x50] { 
        *i = chip8_sprites[pos];
        pos = pos + 1; 
    }

    pos = 0;
    
    for i in &mut memory[0x200..(0x200 + chp8_code.len())] { 
        *i = chp8_code[pos];
        pos = pos+1;
    }
}

fn chp8_dissassemble(chp8_code: &Vec<u8>) -> Result<(), String> {    
    //Memory creation and init
    let mut memory: [u8; 4096];
    memory = [0; 4096];
    init_memory(&chp8_code, &mut memory);

    //Display Memory
    let mut display_mem = [[0u8; 64]; 128];

    //Display Creation and init
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem.window("Chip8", 800, 600)
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas = window.into_canvas().build()
        .expect("could not make a canvas");

    //let mut event_pump = sdl_context.event_pump()?;
    let mut i = 0;

    //println!("mem: {:x} and {:x} ", &memory[0x228], &memory[0x229]);

    let mut reg = Registers::default();
    let mut i = 0;

    loop {
        let pc = reg.PC as usize;    
        let instruction = ((memory[pc] as u16) << 8) | memory[pc+1] as u16;
        let opcode = memory[pc] >> 4;

        let var_nnn = instruction ^ ((opcode as u16) << 12);
        let var_x = (var_nnn >> 8) as u8;
        let var_kk = var_nnn as u8; 
        let var_y = (var_kk >> 4) as u8;
        let var_z = ((var_kk << 4) >> 4) as u8;

        let mut current_key: u8 = 0xff;
        let mut display_updated: bool = false;

        //println!("Instruction: {:x} SP: {:x} PC: {:0>8x} Vx: {:x} Vy: {:x}", instruction,
        //    reg.SP, reg.PC, reg.V[var_x as usize], reg.V[var_y as usize]);

        //println!("var_nnn {:x} var_x {:x} var_y {:x} var_kk {:x}", var_nnn, var_x, var_y, var_kk);

        //println!("mem = {:x}  mem+1 = {:x}  mem+2 = {:x}", memory[pc], memory[pc+1], memory[pc+2]);

        let mut event_pump = sdl_context.event_pump()?;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => break,
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    match keycode {
                        Keycode::Num1 => current_key = 0x01,
                        Keycode::Num2 => current_key = 0x02,
                        Keycode::Num3 => current_key = 0x03,
                        Keycode::Num4 => current_key = 0x0c,
                        Keycode::Q => current_key = 0x04,
                        Keycode::W => current_key = 0x05,
                        Keycode::E => current_key = 0x06,
                        Keycode::R => current_key = 0x0d,
                        Keycode::A => current_key = 0x07,
                        Keycode::S => current_key = 0x08,
                        Keycode::D => current_key = 0x09,
                        Keycode::F => current_key = 0x0e,
                        Keycode::Z => current_key = 0x0a,
                        Keycode::X => current_key = 0x00,
                        Keycode::C => current_key = 0x0b,
                        Keycode::V => current_key = 0x0f,
                        _ => break

                    }
                },
                _ => {}
            }
        }

        //render(&mut canvas, Color::RGB(i, 64, 255 - i));

        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

        match opcode {
            0x00 => {
                if var_kk == 0xe0 {
                    println!("clear screen");
                    display_mem = [[0u8; 64]; 128];
                    canvas.clear();
                    
                    reg.PC = reg.PC + 2;
                } else if var_kk == 0xee {
                    reg.PC = ((memory[reg.SP as usize] as u16) << 8) | (memory[(reg.SP+1) as usize] as u16);
                    //println!("ret to {:x}", reg.PC);
                    reg.SP = reg.SP - 2;
                } else {
                    reg.PC = reg.PC + 2;
                }
            },
            0x01 => {
                reg.PC = var_nnn;
                //break;
            },
            0x02 => {
                
                //something wonky here....
                //println!("Instruction: {:x} SP: {:x} PC: {:0>8x} memory[SP]: {:x}{:x}", instruction,
                //    reg.SP, reg.PC, memory[reg.SP as usize], memory[(reg.SP+1) as usize]);

                //increment stack pointah 
                reg.SP = reg.SP + 2;
               
                memory[reg.SP as usize] = ((reg.PC+2) >> 8) as u8;
                memory[(reg.SP + 1) as usize] =  (reg.PC+2) as u8;
                //println!("SP: {:x} PC: {:0>8x} memory[SP]: {:x}{:x}", 
                //    reg.SP, reg.PC, memory[reg.SP as usize], memory[(reg.SP+1) as usize]);  

                reg.PC = var_nnn; 
            },
            0x03 => {
                //println!("pc {:x} var_x {:x} var_kk {:x}", pc, reg.V[var_x as usize], var_kk);
                if reg.V[var_x as usize] == var_kk {
                    reg.PC = reg.PC + 4;
                } else {
                    reg.PC = reg.PC + 2;
                }
            },
            0x04 => {
                if reg.V[var_x as usize] != var_kk {
                    reg.PC = reg.PC + 4;
                } else {
                    reg.PC = reg.PC + 2;
                }
            },
            0x05 => {
                if var_x == var_y {
                    reg.PC = reg.PC + 4;
                } else {
                    reg.PC = reg.PC + 2;
                }
            },
            0x06 => {
                reg.V[(var_x as usize)] = var_kk; 
                //println!("V {:x} = {:x}", var_x, reg.V[(var_x as usize)]);
                reg.PC = reg.PC + 2;
            },
            0x07 => {                
                //println!("old value for V[{}] is {:x} and kk {:x}", var_x, reg.V[var_x as usize], var_kk);
                reg.V[var_x as usize] = reg.V[var_x as usize].wrapping_add(var_kk);

                reg.PC = reg.PC + 2;
            },
            0x08 => {
                match var_z {
                    0x00 => {
                        reg.V[var_x as usize] = reg.V[var_y as usize];
                        reg.PC = reg.PC + 2;
                    },
                    0x01 => {
                        reg.V[var_x as usize] = reg.V[var_x as usize] | reg.V[var_y as usize];
                        reg.PC = reg.PC + 2; 
                    },
                    0x02 => {
                        reg.V[var_x as usize] = reg.V[var_x as usize] & reg.V[var_y as usize];
                        reg.PC = reg.PC + 2;
                    },
                    0x03 => {
                        reg.V[var_x as usize] = reg.V[var_x as usize] ^ reg.V[var_y as usize];
                        reg.PC = reg.PC + 2;
                    },
                    0x04 => {
                        println!("Instruction: {:x} SP: {:x} PC: {:0>8x} x-value {} y-value {}", instruction,
                                reg.SP, reg.PC, reg.V[var_x as usize], reg.V[var_y as usize]);
                        
                        let v_x = reg.V[var_x as usize] as u16;
                        let v_y = reg.V[var_y as usize] as u16;

                        //let temp: u16 = reg.V[var_x as usize].wrapping_add(reg.V[var_y as usize]) as u16;
                        
                        let temp = v_x.wrapping_add(v_y);
                        let temp2 = (reg.V[var_x as usize] as u32 + reg.V[var_y as usize] as u32) as u32;

                        reg.V[var_x as usize] = temp as u8;

                        println!("sum of operation {} {}", temp, temp2);

                        if temp2 > 255 {
                            reg.V[0xF] = 1;
                        } else {
                            reg.V[0xF] = 0;
                        }
                        
                        reg.PC = reg.PC + 2;

                        println!("8x4 VF value {}", reg.V[0xF]);
                    },
                    0x05 => {
                        println!("Instruction: {:x} SP: {:x} PC: {:0>8x} x-value {} y-value {}", instruction,
                                reg.SP, reg.PC, reg.V[var_x as usize], reg.V[var_y as usize]);

                        //println!("8x5 compare {}", reg.V[var_x as usize] > reg.V[var_y as usize]);
                        let v_x = reg.V[var_x as usize];
                        let v_y = reg.V[var_y as usize];

                        let temp = v_x.wrapping_sub(v_y);

                        reg.V[var_x as usize] = temp;

                        if v_x >= v_y {
                            reg.V[0xF] = 1;
                        } else {
                            reg.V[0xF] = 0;
                        }

                        reg.PC = reg.PC + 2;
                        println!("8x5 VF value {} and vx {} and vy {} and v[x] {}", reg.V[0xF], v_x, v_y, reg.V[var_x as usize]);
                    },
                    0x06 => {
                        //println!("Instruction: {:x} SP: {:x} PC: {:0>8x} x-value {} y-value {}", instruction,
                        //        reg.SP, reg.PC, reg.V[var_x as usize], reg.V[var_y as usize]);

                        //println!("lsb value: {:x}", reg.V[var_x as usize] & 1);
                        let v_x = reg.V[var_x as usize];
                        let v_y = reg.V[var_y as usize];

                        let temp = reg.V[var_x as usize] / 2;
                        reg.V[var_x as usize] = temp;

                        if (v_x & 1) == 1 {
                            reg.V[0xF] = 1;
                        } else {
                            reg.V[0xF] = 0;
                        }
                        reg.PC = reg.PC + 2;
                        //println!("8x6 VF value {}", reg.V[0xF]);
                    },
                    0x07 => {
                        //println!("Instruction: {:x} SP: {:x} PC: {:0>8x} x-value {} y-value {}", instruction,
                        //       reg.SP, reg.PC, reg.V[var_x as usize], reg.V[var_y as usize]);
                        let v_x = reg.V[var_x as usize];
                        let v_y = reg.V[var_y as usize];

                        let temp = reg.V[var_y as usize].wrapping_sub(reg.V[var_x as usize]);
                        reg.V[var_x as usize] = temp;

                        if v_y >= v_x {
                            reg.V[0xF] = 1;
                        } else {
                            reg.V[0xF] = 0;
                        }
                        
                        reg.PC = reg.PC + 2;
                        //println!("8x7 VF value {}", reg.V[0xF]);
                    },
                    0x0E => {
                        //println!("Instruction: {:x} SP: {:x} PC: {:0>8x} x-value {} y-value {}", instruction,
                        //        reg.SP, reg.PC, reg.V[var_x as usize], reg.V[var_y as usize]);

                        //println!("8xE and operation {:x}", reg.V[var_x as usize] & 0x80);
                        let v_x = reg.V[var_x as usize];
                        let v_y = reg.V[var_y as usize];
                        
                        let temp = reg.V[var_x as usize].wrapping_mul(2);
                        reg.V[var_x as usize] = temp;

                        if (v_x & 0x80) == 0x80 {
                            reg.V[0xF] = 1;
                        } else {
                            reg.V[0xF] = 0;
                        }
                        
                        reg.PC = reg.PC + 2;
                        //println!("8xE VF value {}", reg.V[0xF]);
                    },
                    _ => {
                        reg.PC = reg.PC + 2;
                    },
                }
            },
            0x09 => {
                if reg.V[var_x as usize] != reg.V[var_y as usize] {
                    reg.PC = reg.PC + 4;
                } else {
                    reg.PC = reg.PC + 2;
                }
            },
            0x0a => {
                reg.I = var_nnn;
                reg.PC = reg.PC + 2;
                //println!("opcode {:x} variable {:x}", opcode, reg.I);
            },
            0x0b => {
                reg.PC = var_nnn + (reg.V[0] as u16);
            },
            0x0c => {
                let mut rng = rand::thread_rng();

                let rnd: u8 = rng.gen();
                reg.V[var_x as usize] = rnd & var_kk;

                reg.PC = reg.PC + 2;
            },
            0x0d => {
                reg.PC = reg.PC + 2;

                update_display_mem(&mut display_mem, &mut reg, (var_x as usize), 
                    (var_y as usize), &mut memory, (var_z as usize));
                
                display(&mut canvas, &mut reg, (var_x as usize), 
                    (var_y as usize), &mut memory, (var_z as usize)); 
                
                //println!("display at X: {} Y: {} the following: {:b}", 
                //    reg.V[(var_x as usize)], reg.V[(var_y as usize)], 
                //    memory[(reg.I as usize)]);
            },
            0x0e => {
                if var_kk == 0x9e {
                    if reg.V[var_x as usize] == current_key {
                        reg.PC = reg.PC + 4;
                    } else {
                        reg.PC = reg.PC + 2;
                    }
                } else if var_kk == 0xa1 {
                    if reg.V[var_x as usize] != current_key {
                        reg.PC = reg.PC + 4;
                    } else {
                        reg.PC = reg.PC + 2;
                    }
                } else {
                    reg.PC = reg.PC + 2;
                }
            },
            0x0f => {
                match var_kk {
                    0x07 => {
                        reg.V[var_x as usize] = reg.DT;
                        reg.PC = reg.PC + 2;
                    },
                    0x0a => {
                        reg.V[var_x as usize] = current_key;
                        if current_key != 0xff {
                            reg.PC = reg.PC + 2;
                        }
                    },
                    0x15 => {
                        reg.DT = reg.V[var_x as usize];
                        reg.PC = reg.PC + 2;
                    },
                    0x18 => {
                        reg.ST = reg.V[var_x as usize];
                        reg.PC = reg.PC + 2;
                    },
                    0x1e => {
                        reg.I = reg.I + (reg.V[var_x as usize] as u16);
                        reg.PC = reg.PC + 2;
                    },
                    0x29 => {
                        reg.I = memory[(var_x*5) as usize] as u16;
                        reg.PC = reg.PC + 2;
                    },
                    0x33 => {
                        let mut dec: u8 = reg.V[var_x as usize];

                        memory[(reg.I+2) as usize] = dec % 10;
                        dec = dec / 10;

                        memory[(reg.I+1) as usize] = dec % 10;
                        dec = dec / 10;

                        memory[reg.I as usize] = dec % 10;

                        reg.PC = reg.PC + 2;
                    },
                    0x55 => {
                        for i in 0..=var_x {
                            memory[(reg.I + (i as u16)) as usize] = reg.V[i as usize];
                        }
                        reg.PC = reg.PC + 2;
                    },
                    0x65 => {
                        for i in 0..=var_x {
                            reg.V[i as usize] = memory[(reg.I + (i as u16)) as usize];
                        }
                        reg.PC = reg.PC + 2;
                    },
                    _ => {
                        println!("UNKNOWN 0xF INSTRUCTION {:x} at position {:x}", instruction, reg.PC);
                        reg.PC = reg.PC + 2;
                    }
                }
            },
            _ => { 
                reg.PC = reg.PC + 2;
            },
        }

    }

}
