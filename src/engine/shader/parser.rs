use std::process::id;


#[derive(Debug)]
struct Op {
    pub opcode: u32,
    pub offset: u32,
    pub size: u32
}

pub struct Parser {

}

impl Parser{
    pub fn new(spv_data: &Vec<u32>) -> Parser {
        assert!(spv_data[0] == 0x07230203, "Not valid SPIRV file, magic number not there");

        Parser::process_data(spv_data);

        Parser {  }
    }

    fn process_data(spv_data: &Vec<u32>) {
        let id_bound = spv_data[3];

        let mut ops = Vec::new();

        let mut i = 5u32;

        while i != spv_data.len() as u32 {
            let offset = i;
            let opcode = spv_data[i as usize] & 0xff;
            let size = spv_data[i as usize] >> 16;
            //2.3 spirv spec

            ops.push(
                Op {
                    offset,
                    opcode,
                    size
                });

            i += size;
        }

        dbg!(&ops);
    }
}