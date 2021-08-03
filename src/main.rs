use std::env;
use std::fs;
use std::thread;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::WindowCanvas;
use std::time::Duration;


struct Registers {
    V: [u8; 16],
    I: u16,
    SP: u16,
    PC: u16,
}

impl Default for Registers {
    fn default() -> Registers {
        Registers { 
            I: 0,
            V: [0; 16],
            SP: 0xfa0,
            PC: 0x200
        }
    }
}


fn get_bit(byte: u8, pos: u8) -> u8 {
    if(pos == 8) {
        return (byte << 7) & 0x1;
    } 
    
   return ((byte >> (8-pos)) & 0x1) as u8;
}

fn display(canvas: &mut WindowCanvas, reg: &mut Registers, X: usize, 
    Y: usize, memory: &mut [u8; 4096], N: usize) {
   
   let mut x_pos = (reg.V[X] as i32)*8;
   let mut y_pos = (reg.V[Y] as i32)*8;
   let index = reg.I as usize; 
   println!("display at X: {} and Y: {} this sprite info: {:x?}", reg.V[X], reg.V[Y],
        &memory[index..(index+N)]);
    
    for byte in &memory[index..(index+N)] {
        for i in 1..=8 {
            if get_bit(*byte, i) == 1 {
                canvas.set_draw_color(Color::RGB(255,255,25));
                canvas.fill_rect(Rect::new(x_pos, y_pos, 8, 8));
                canvas.present();
            } else {
                canvas.set_draw_color(Color::RGB(0,0,0));
                canvas.fill_rect(Rect::new(x_pos, y_pos, 8, 8));
                canvas.present();
            }
            thread::sleep_ms(10);
            x_pos = x_pos + 8;
        }
        x_pos = (reg.V[X] as i32)*8;
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
    for i in &mut memory[0x0..0x200] { *i = 0x0 }
    
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

    println!("mem: {:x} and {:x} ", &memory[0x228], &memory[0x229]);

    let mut reg = Registers::default();
    let mut i = 0;

    loop {
        let pc = reg.PC as usize;    
        let instruction = ((memory[pc] as u16) << 8) | memory[pc+1] as u16;
        let opcode = memory[pc] >> 4;
        let var = instruction ^ ((opcode as u16) << 12);

        println!("pc {:x} instruction {:x} opcode {:x} var {:x}", 
            pc, instruction, opcode, var);

        /*for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break;
                },
                _ => {}
            }
        }*/

        //render(&mut canvas, Color::RGB(i, 64, 255 - i));

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));

        match opcode {
            0x01 => {
                reg.PC = var;
                //break;
            },
            0x03 => {
                println!("pc {:x} op {:x} var {:x}", pc, opcode, var);
                reg.PC = reg.PC + 2;
            },
            0x06 => {
                let var2 = var >> 8;

                reg.V[(var2 as usize)] = (((var2 << 8) as u16) ^ var) as u8; 
                println!("V {:x} = {:x}", var2, reg.V[(var2 as usize)]);
                reg.PC = reg.PC + 2;
            },
            0x07 => {
                let var_i = (var >> 8) as usize;
                let var2 = (((var >> 8) << 8) ^ var) as u8;  
                
                println!("old value for V[{}] is {:x}", var_i, reg.V[var_i]);
                reg.V[var_i] = reg.V[var_i] + var2;

                reg.PC = reg.PC + 2;

                println!("new V[{}] is {:x}", var_i, reg.V[var_i]);
            },
            0x0a => {
                reg.I = var;
                reg.PC = reg.PC + 2;
                println!("opcode {:x} variable {:x}", opcode, reg.I);
            },
            0x0d => {
                let var1 = var >> 8;
                let var2 = (var >> 4) ^ (var1 << 4);

                let mut N = (var << 12);
                N = (N >> 12);

                reg.PC = reg.PC + 2;
                
                display(&mut canvas, &mut reg, (var1 as usize), 
                    (var2 as usize), &mut memory, (N as usize)); 
                
                println!("display at X: {} Y: {} the following: {:b}", 
                    reg.V[(var1 as usize)], reg.V[(var2 as usize)], 
                    memory[(reg.I as usize)]);
            },
            _ => { 
                reg.PC = reg.PC + 2;
            },
        }

    }

}
