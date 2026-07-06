use crate::dex::parser::DexDocument;

pub fn decode_instruction(insns: &[u16], pc: usize, dex: &DexDocument) -> usize {
    let ins = insns[pc];
    let opcode = (ins & 0xff) as u8;
    //let register = (ins >> 8) as u8;
    match opcode {
        0x1a => {
            let reg = (ins >> 8) as u8;
            let idx = insns[pc + 1] as usize;
            println!("const-string v{}, string@{} \"{}\"", reg, idx, dex.strings.strings[idx]);
            2
        }

        0x71 => {
            let method_idx = insns[pc+1] as usize;
            let regs = decode_35c_registers(ins, insns[pc + 2]);
            println!("invoke-static {:?}, {}", regs, resolve_method(method_idx,dex));
            3
        }

        0x6e => {
            let method_idx = insns[pc+1] as usize;
            println!("{:04x} invoke-virtual {}", ins, resolve_method(method_idx,dex));
            3
        }

        0x38 => {
            println!("{:04x} if-eqz", ins);
            2
        }
        0x10 => {
            println!("{:04x} const/16", ins);
            2
        }

        0x11 => {
            println!("{:04x} const/high16", ins);
            2
        }

        0x1f => {
            let reg = (ins >> 8) as u8;
            let type_idx = insns[pc + 1] as usize;
            let typ = &dex.type_ids.types[type_idx];
            println!("check-cast v{}, {}", reg, dex.strings.strings[typ.descriptor_idx as usize]);
            2
        }

        0x20 => {
            println!("{:04x} instance-of", ins);
            2
        }

        0x21 => {
            println!("{:04x} array-length", ins);
            1
        }

        0x28 => {
            println!("{:04x} goto", ins);
            1
        }

        0x2b => {
            println!("{:04x} packed-switch", ins);
            3
        }

        0x54 => {
            println!("{:04x} iget-object", ins);
            2
        }

        0x60 => {
            let field_idx = insns[pc + 1] as usize;
            println!("{:04x} sget {}", ins, resolve_field(field_idx, dex));
            2
        }

        0x62 => {
            println!("{:04x} sget-object", ins);
            2
        }

        0x69 => {
            println!("{:04x} sput-object", ins);
            2
        }

        0x70 => {
            let method_idx = insns[pc+1] as usize;
            println!("{:04x} invoke-direct {}", ins, resolve_method(method_idx,dex));
            3
        }

        0x72 => {
            let method_idx = insns[pc+1] as usize;
            println!("invoke-interface {}", resolve_method(method_idx,dex));
            3
        }
        
        0x27 => {
            println!("throw");
            1
        }

        0x0e => {
            println!("return-void");
            1
        }

        0x0f => {
            println!("return");
            1
        }

        0x12 => {
            println!("const/4");
            1
        }
    
        0x5b => {
            println!("{:04x} iput-object", ins);
            2
        }

        0x6f => {
            let method_idx = insns[pc + 1] as usize;
            println!("{:04x} invoke-super {}", ins, resolve_method(method_idx, dex));
            3
        }

        0x22 => {
            let type_idx = insns[pc + 1] as usize;
            let typ = &dex.type_ids.types[type_idx];
            println!("{:04x} new-instance {}", ins, dex.strings.strings[typ.descriptor_idx as usize]);
            2
        }

        0x0c => {
            let reg = (ins >> 8) as u8;

            println!("move-result-object v{}", reg);
            1
        }

        _ => {
            println!("{:04x} unknown", ins);
            1
        }
    }
}

fn resolve_method(idx: usize, dex: &DexDocument) -> String {
    let method = &dex.method_ids.methods[idx];
    let class = &dex.type_ids.types[method.class_idx as usize];
    let class_name = &dex.strings.strings[class.descriptor_idx as usize];
    let name = &dex.strings.strings[method.name_idx as usize];
    
    format!(
        "{}->{}",
        class_name,
        name
    )
}

fn resolve_field(idx: usize, dex: &DexDocument) -> String {
    let field = &dex.field_ids.fields[idx];
    let class = &dex.type_ids.types[field.class_idx as usize];
    let class_name = &dex.strings.strings[class.descriptor_idx as usize];
    let name = &dex.strings.strings[field.name_idx as usize];
    format!(
        "{}->{}",
        class_name,
        name
    )
}

fn decode_35c_registers(first: u16, third: u16) -> Vec<u8> {
    let count = (first >> 12) as usize;
    let c = (third & 0xf) as u8;
    let d = ((third >> 4) & 0xf) as u8;
    let e = ((third >> 8) & 0xf) as u8;
    let f = ((third >> 12) & 0xf) as u8;
    let regs = [c,d,e,f];

    regs[..count].to_vec()
}
