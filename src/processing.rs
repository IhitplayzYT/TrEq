pub mod process{
    use std::sync::{Arc, Mutex};

use ringbuf::traits::Consumer;

use crate::models::models::App;


    pub fn process_mono_audio(app: Arc<Mutex<App>>, cons: Arc<Mutex<impl Consumer<Item = f32> + Send>>, sample_rate: u32, channels: u16,batch_sz:usize){
    }

    pub fn process_stereo_audio(app: Arc<Mutex<App>>, lcons: Arc<Mutex<impl Consumer<Item = f32> + Send>>,rcons: Arc<Mutex<impl Consumer<Item = f32> + Send>>, sample_rate: u32, channels: u16,batch_sz:usize){
    }



}