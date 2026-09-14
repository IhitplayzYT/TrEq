use std::path::PathBuf;

use crate::{helper::Helper::CLI, models::models::App, processing::process::{process_mono_audio, process_stereo_audio}};

mod helper;
mod render;
mod input;
mod filters;
mod processing;
mod models;
mod Dao;
use std::{cell::LazyCell, error::Error, io::{self, stdout}, sync::{Arc, LazyLock, Mutex, atomic::{AtomicBool, Ordering}}, time::Duration};
use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}};
use crossterm::{event::{DisableMouseCapture, EnableMouseCapture}, execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}};
use ratatui::{Terminal, backend::CrosstermBackend};
use ringbuf::{HeapRb, traits::{Consumer, Observer, Producer, Split}};
pub const FRAME_SIZE:usize = 1024;

static running:LazyLock<Arc<AtomicBool>> = LazyLock::new(|| Arc::new(AtomicBool::new(true)));
static r: LazyLock<Arc<AtomicBool>> = LazyLock::new(|| running.clone());

pub fn add_sigint_handler(){
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).unwrap();
}

#[derive(Debug)]
struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>>{
    add_sigint_handler();
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore_terminal(); 
        original_hook(panic_info);
    }));

    let mut clargs = CLI::new();
    clargs.Parse_Args();

    if clargs.dbg{
        println!("{clargs:?}");
    }

    let app = App::new(clargs.output_profiles_path);
    
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout,EnterAlternateScreen,EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    let mut terminal = TerminalGuard {terminal};
    
    let host = cpal::host_from_id(render::render::select_host(&mut terminal.terminal)?)?;
    let input_device_name = render::render::select_device(&mut terminal.terminal, &host)?;
    let output_device_name = render::render::select_output_device(&mut terminal.terminal, &host)?;
    
    let mut output_stream = None;
    let mut stream = None;
    let mut sample_rate = 48000_u32;
    let mut channels = 2;
    let RING_BUFF_SZ = sample_rate as usize * 2;

    let proc_buff = HeapRb::<f32>::new(RING_BUFF_SZ);
    let (lbuff,rbuff) = (HeapRb::<f32>::new(RING_BUFF_SZ/2),HeapRb::<f32>::new(RING_BUFF_SZ/2));
    let (loutbuff, routbuff) = (HeapRb::<f32>::new(RING_BUFF_SZ/2), HeapRb::<f32>::new(RING_BUFF_SZ/2));
    let outbuff = HeapRb::<f32>::new(RING_BUFF_SZ);

    let (mut lprod,lcons) = lbuff.split();
    let (mut rprod,rcons) = rbuff.split();
    let (mut loutprod, loutcons) = loutbuff.split();
    let (mut routprod, routcons) = routbuff.split();
    let (mut prod,cons) = proc_buff.split();
    let (mut outprod, outcons) = outbuff.split();
    
    let lcons = Arc::new(Mutex::new(lcons));
    let rcons = Arc::new(Mutex::new(rcons));
    let cons = Arc::new(Mutex::new(cons));
    let loutprod = Arc::new(Mutex::new(loutprod));
    let routprod = Arc::new(Mutex::new(routprod));
    let outprod = Arc::new(Mutex::new(outprod));
    let loutcons = Arc::new(Mutex::new(loutcons));
    let routcons = Arc::new(Mutex::new(routcons));
    let outcons = Arc::new(Mutex::new(outcons));
    
    for dev in host.input_devices().unwrap(){       
        let name = dev.to_string();
        if name == input_device_name {
            let conf = dev.default_input_config().expect("No config for the input device");
            let conf: cpal::StreamConfig = conf.clone().into();
            sample_rate = conf.sample_rate;
            channels = conf.channels;
            assert!(channels <= 2);
            stream = Some(dev.build_input_stream(conf, move |data:&[f32],_|{
                if clargs.stereo{
                    for (idx,v) in data.iter().enumerate(){
                        if idx & 1 == 0{
                            let _ = lprod.try_push(*v);
                        }else{
                            let _ = rprod.try_push(*v);
                        }
                    }
                } else{
                    for i in data.chunks_exact(2){
                        let _ = prod.try_push((i[0] + i[1]) * 0.5);
                    }
                }
            }, move |err| {panic!("Exiting due to {err}");}, None).unwrap());
            break;
        }
    }

    if let Some(strm) = stream{
        let app = Arc::new(Mutex::new(app));
        let app_clone = app.clone();
        
        let output_device = host.output_devices().unwrap().find(|d| d.to_string() == output_device_name).expect("Output device not found");
        let output_config = output_device.default_output_config().expect("No output config");
        let output_config: cpal::StreamConfig = output_config.clone().into();
        
        if clargs.stereo{
            let lcons_clone = lcons.clone();
            let rcons_clone = rcons.clone();
            let loutprod_clone = loutprod.clone();
            let routprod_clone = routprod.clone();
            std::thread::spawn(move || {process_stereo_audio(app_clone, lcons_clone, rcons_clone, loutprod_clone, routprod_clone, sample_rate, channels,64);});
            output_stream = Some(output_device.build_output_stream(output_config,
        move |data: &mut [f32], _| {
                    let mut l_consumer = loutcons.lock().unwrap();
                    let mut r_consumer = routcons.lock().unwrap();
                    for i in data.chunks_exact_mut(2) {
                        i[0] = l_consumer.try_pop().unwrap_or(0.0);
                        i[1] = r_consumer.try_pop().unwrap_or(0.0);
                    }
                },
                move |err| {panic!("Output stream error: {}", err)},
                None
            ).unwrap());
        }else{
            let cons_clone = cons.clone();
            let outprod_clone = outprod.clone();
            std::thread::spawn(move || {process_mono_audio(app_clone, cons_clone, outprod_clone, sample_rate, channels,64);});
            
            output_stream = Some(output_device.build_output_stream(output_config,
                move |data: &mut [f32], _| {
                    let mut consumer = outcons.lock().unwrap();
                    for i in data.iter_mut() {
                        *i = consumer.try_pop().unwrap_or(0.0);
                    }
                },
                move |err| {panic!("Output stream error: {}", err)},
                None
            ).unwrap());
        }
        
        strm.play()?;
        if let Some(out_strm) = output_stream {
            out_strm.play().unwrap();
        }

        std::thread::sleep(Duration::from_millis(100));
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout,EnterAlternateScreen,EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        let mut terminal = TerminalGuard {terminal};
        render::render::run_app(&mut terminal.terminal, app)?;
        Ok(())
        
    }else{
        panic!("No valid virtual stream found on host device");
    }
}
