pub fn find_node(&mut self, look_for: &str) -> Result<*mut usize, ()> {
        self.offset = self.base + self.header.as_mut().unwrap().off_dt_struct as usize;
        let mut level: u32 = 0;
        loop {
            let token = self.read_u32();
            for _ in 0..level {
                kprint!("|");
            }
            kprint!("- ");
            match token {
                0x1 => {
                    kprintln!("NODE: {}", self.read_nullterm_string());
                    level += 1;
                }
                0x2 => {level -= 1;}
                0x3 => {
                    let prop_header: FDTProp = FDTProp {
                        length: self.read_u32(),
                        name_offset: self.read_u32(),
                    };
                    {
                        let value = self.grab_string(prop_header.name_offset);
                        kprint!("prop `{}`: ", value);
                    }
                    let property = self.read_string(prop_header.length);
                    kprint!("`");
                    match core::str::from_utf8(property) {
                        Ok(s) => {kprint!("{}", s);}
                        Err(b) => {
                            for x in property {
                                kprint!("{:x} ", x);
                            }
                        }
                    }
                    kprint!("`");
                    kprintln!();
                }
                0x4 => {}
                0x9 => break,
                _ => {
                    panic!("Invalid FDT lexical token of 0x{:X}", token);
                }
            }
        }
        Err(())
    }