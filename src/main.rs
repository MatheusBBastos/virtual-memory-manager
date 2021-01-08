use std::{collections::VecDeque, fs::File, io::{BufReader, SeekFrom}};
use std::str;
use std::io::prelude::*;
use std::env;

const PAGE_SIZE: usize = 256;
const NUM_PAGES: usize = 256;
const NUM_FRAMES: usize = 128; // Originalmente 256 (para obter o resultado de correct.txt)
const MEM_SIZE: usize = PAGE_SIZE * NUM_FRAMES;
const TLB_ENTRIES: usize = 16;

const BACKING_STORE_PATH: &str = "BACKING_STORE.bin";

/// Estrutura que representa uma memória
struct Memory {
    data: [u8; MEM_SIZE],
    page_table: PageTable,
    tlb: VecDeque<Entry>,
}

/// Estrutura usada para mapear uma página a um frame
struct Entry {
    pg_num: u32,
    frame_num: u32,
}

/// Estrutura que representa uma tabela de páginas
struct PageTable {
    frame_nums: [Option<u32>; NUM_PAGES],
    swap_queue: VecDeque<Entry>,
}

/// Estrutura que representa o resultado de uma consulta na memória
struct QueryResult {
    physical_addr: usize,
    page_fault: bool,
    tlb_hit: bool,
    value: i8,
}

impl Memory {
    /// Inicializa uma memória
    pub fn new() -> Memory {
        Memory {
            data: [0; MEM_SIZE],
            page_table: PageTable::new(),
            tlb: VecDeque::with_capacity(TLB_ENTRIES),
        }
    }

    /// Consulta o TLB, retornando o frame correspondente se a página estiver armazenada nele
    fn consult_tlb(&self, pg_num: u32) -> Option<u32> {
        for tlb_entry in self.tlb.iter() {
            if tlb_entry.pg_num == pg_num {
                return Some(tlb_entry.frame_num);
            }
        }

        None
    }

    /// Insere um mapeamento página-frame no TLB
    fn update_tlb(&mut self, pg_num: u32, frame_num: u32) {
        if self.tlb.len() == TLB_ENTRIES {
            // Fila do TLB está cheia, remover página mais antiga (inserida antes)
            self.tlb.pop_front();
        }

        // Inserir nova página
        self.tlb.push_back(Entry {
            pg_num,
            frame_num,
        });
    }

    /// Lê a página `pg_num` do arquivo `bck_store` e a armazena no frame `frame_num`
    fn read_from_file(&mut self, pg_num: u32, frame_num: usize, bck_store: &mut File) {
        let frame_end = frame_num + PAGE_SIZE;

        bck_store.seek(SeekFrom::Start((pg_num * PAGE_SIZE as u32) as u64))
            .expect("Falha ao posicionar cursor no arquivo");
        bck_store.read(&mut self.data[frame_num..frame_end])
            .expect("Falha ao ler arquivo");
    }

    /// Consulta a memória usando o endereço virtual `virtual_addr` e o arquivo
    /// `bck_store` como base
    pub fn query(&mut self, virtual_addr: u32, bck_store: &mut File) -> QueryResult {
        // Extrair os 8 primeiros bits do endereço (número da página)
        let pg_num = virtual_addr >> 8;
        // Extrair os 8 últimos bits do endereço (deslocamento)
        let offset = (virtual_addr & 0xFF) as usize;

        if let Some(frame_num) = self.consult_tlb(pg_num) {
            // TLB hit

            let physical_addr = frame_num as usize + offset;
                
            QueryResult {
                physical_addr,
                page_fault: false,
                tlb_hit: true,
                value: self.data[physical_addr] as i8,
            }
        } else if let Some(frame_num) = self.page_table.frame_nums[pg_num as usize] {
            // Page hit

            self.update_tlb(pg_num, frame_num as u32);

            let physical_addr = frame_num as usize + offset;

            QueryResult {
                physical_addr,
                page_fault: false,
                tlb_hit: false,
                value: self.data[physical_addr] as i8,
            }
        } else {
            // Page miss

            let frame_num = self.page_table.get_frame_num(pg_num);

            self.update_tlb(pg_num, frame_num as u32);
            self.read_from_file(pg_num, frame_num, bck_store);

            let physical_addr = frame_num + offset;
            
            QueryResult {
                physical_addr,
                page_fault: true,
                tlb_hit: false,
                value: self.data[physical_addr] as i8,
            }
        }
    }
}

impl PageTable {
    /// Inicializa uma tabela de páginas
    pub fn new() -> PageTable {
        PageTable {
            frame_nums: [None; NUM_PAGES],
            swap_queue: VecDeque::with_capacity(NUM_FRAMES),
        }
    }

    /// Obtém o número do frame correspondente à página `pg_num`
    pub fn get_frame_num(&mut self, pg_num: u32) -> usize {
        let frame_num = if self.swap_queue.len() == NUM_FRAMES {
            // Memória está cheia, remover página mais antiga e usar o seu frame
            let swapped_page = self.swap_queue.pop_front().unwrap();
            self.frame_nums[swapped_page.pg_num as usize] = None;
            swapped_page.frame_num as usize
        } else {
            self.swap_queue.len() * PAGE_SIZE
        };

        self.swap_queue.push_back(Entry { pg_num, frame_num: frame_num as u32});
        self.frame_nums[pg_num as usize] = Some(frame_num as u32);

        frame_num
    }
}

fn main() -> std::io::Result<()> {
    let mut bck_store = File::open(BACKING_STORE_PATH).expect("Arquivo backing store não encontrado");

    let path = env::args().nth(1).expect("Informe um arquivo");
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);

    let mut memory = Memory::new();

    let mut page_faults = 0;
    let mut tlb_hits = 0;
    let mut count = 0;

    loop {
        let mut addr = String::new();
        let bytes = buf_reader.read_line(&mut addr).expect("Falha ao ler arquivo");

        // Fim de arquivo atingido
        if bytes == 0 {
            break;
        }

        count += 1;

        let addr: u32 = addr.trim().parse().expect("Número inválido");
        let addr_masked = addr & 0xFFFF;
        let query_result = memory.query(addr_masked, &mut bck_store);

        if query_result.page_fault {
            page_faults += 1;
        }

        if query_result.tlb_hit {
            tlb_hits += 1;
        }

        print!("Virtual address: {} ", addr_masked);
        print!("Physical address: {} ", query_result.physical_addr);
        println!("Value: {}", query_result.value);
    }

    println!("Number of Translated Addresses = {}", count);
    println!("Page Faults = {}", page_faults);
    println!("Page Fault Rate = {}", page_faults as f64 / count as f64);
    println!("TLB Hits = {}", tlb_hits);
    println!("TLB Hit Rate = {}", tlb_hits as f64 / count as f64);
    
    Ok(())
}
