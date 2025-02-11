use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering}};
use std::process;
use std::io::{self, Write};
use bip39::{Mnemonic, Language};
use ctrlc;
use rustacuda::launch;
use rustacuda::memory::DeviceBuffer;
use rustacuda::prelude::*;
use log::{info, error};
use std::ffi::CString;
use crossbeam::scope;
extern crate num_cpus;

fn main() {
    env_logger::init();

    let num_cpus = num_cpus::get();
    info!("Количество логических процессоров: {}", num_cpus);

    print!("Введите количество используемых процессоров: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let used_cpus: usize = input.trim().parse().unwrap_or(num_cpus);
    let _used_cpus = used_cpus.min(num_cpus);

    let known_words = "forum kitchen recall child zoo memory";
    let known_words: Vec<&str> = known_words.split_whitespace().collect();

    let target_address = "TGaVQqjm4zxYFkT1SM9P8frBA38jWdZR4D";

    let found_phrase = Arc::new(Mutex::new(None));
    let progress_counter = Arc::new(AtomicUsize::new(0));
    let total_combinations = 2048u64.pow(4);

    ctrlc::set_handler(move || {
        println!("Программа завершена пользователем.");
        process::exit(0);
    }).expect("Ошибка при настройке обработчика Ctrl+C");

    rustacuda::init(CudaFlags::empty()).unwrap();

    let combinations_per_gpu = total_combinations / 4;

    scope(|s| {
        for gpu_id in 0..4 {
            let found_phrase = Arc::clone(&found_phrase);
            let known_words = known_words.clone();
            let progress_counter = Arc::clone(&progress_counter);

            s.spawn(move |_| {
                let device = Device::get_device(gpu_id).unwrap();
                let _context = Context::create_and_push(ContextFlags::empty(), device).unwrap();

                let ptx_filename = CString::new("kernel.ptx").unwrap();
                let ptx_module = Module::load_from_file(&ptx_filename).unwrap();
                let check_combinations = ptx_module.get_function(&CString::new("check_combinations").unwrap()).unwrap();

                let start_idx = (gpu_id as u64) * combinations_per_gpu;
                let end_idx = ((gpu_id + 1) as u64) * combinations_per_gpu;


                let known_words_str = known_words.join(" ");
                let known_words_bytes = known_words_str.as_bytes().to_vec();

                let mut known_words_device = DeviceBuffer::from_slice(&known_words_bytes).unwrap();
                let mut target_address_device = DeviceBuffer::from_slice(target_address.as_bytes()).unwrap();
                let mut found_phrase_device = DeviceBuffer::from_slice(&vec![0u8; 128]).unwrap();
                let mut progress_counter_device = DeviceBuffer::from_slice(&vec![0u64]).unwrap();

                let stream = Stream::new(StreamFlags::NON_BLOCKING, None).unwrap();

                unsafe {
                    launch!(check_combinations<<<128, 128, 0, stream>>>(
                        known_words_device.as_device_ptr(),
                        target_address_device.as_device_ptr(),
                        found_phrase_device.as_device_ptr(),
                        progress_counter_device.as_device_ptr(),
                        start_idx as u64,
                        end_idx as u64
                    )).unwrap();
                }

                let mut found_phrase_host = vec![0u8; 128];
                let mut progress_counter_host = vec![0u64];

                found_phrase_device.copy_to(&mut found_phrase_host[..]).unwrap();
                progress_counter_device.copy_to(&mut progress_counter_host[..]).unwrap();

                progress_counter.fetch_add(progress_counter_host[0] as usize, Ordering::Relaxed);

                if found_phrase_host[0] != 0 {
                    let mut found = found_phrase.lock().unwrap();
                    *found = Some(String::from_utf8(found_phrase_host).unwrap());
                }
            });
        }
    }).unwrap();

   

    {
        let locked_phrase = found_phrase.lock().unwrap();
        if let Some(phrase) = &*locked_phrase {
            let words: Vec<&str> = phrase.split_whitespace().collect();
            let last_four_words = &words[words.len() - 4..];
    
            info!("Последние 4 слова: {}", last_four_words.join(" "));
    
            let mnemonic = Mnemonic::parse_in_normalized(Language::English, phrase).unwrap();
            info!("Найдена сид-фраза: {}", mnemonic.to_string());
    
            info!("Работа завершена успешно.");
        } else {
            error!("Сид-фраза не найдена.");
        }
    } // Здесь `locked_phrase` выходит из области видимости, и мьютекс разблокируется
}
