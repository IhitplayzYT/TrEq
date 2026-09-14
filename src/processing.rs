pub mod process{
    use std::sync::{Arc, Mutex};
    use std::collections::HashMap;
    use ringbuf::traits::{Consumer, Producer};
    use uuid::Uuid;
    use crate::models::models::{App, EqProfile, EqNode, EqBand, EqFilter};
    use crate::filters::filters::Biquad;


    pub fn process_mono_audio(app: Arc<Mutex<App>>, cons: Arc<Mutex<impl Consumer<Item = f32> + Send>>, prod: Arc<Mutex<impl Producer<Item = f32> + Send>>, sample_rate: u32, _channels: u16,batch_sz:usize){
        let mut biquad_cache: HashMap<(Uuid,Uuid), Biquad> = HashMap::new();
        loop {
            let mut samples = Vec::with_capacity(batch_sz);
            let mut consumer = cons.lock().unwrap();

            for _ in 0..batch_sz {
                if let Some(sample) = consumer.try_pop() {
                    samples.push(sample);
                }
            }
            
            if samples.is_empty() {
                std::thread::sleep(std::time::Duration::from_micros(100));
                continue;
            }

            let app_guard = app.lock().unwrap();
            if let Some(profile) = app_guard.profiles.get(app_guard.selected_profile_idx) {
                let processed = apply_eq_profile(&samples, profile, sample_rate as usize, &mut biquad_cache);
                drop(app_guard);
                
                let mut producer = prod.lock().unwrap();
                for sample in processed {
                    let _ = producer.try_push(sample);
                }
            }
        }
    }

    pub fn process_stereo_audio(app: Arc<Mutex<App>>, lcons: Arc<Mutex<impl Consumer<Item = f32> + Send>>,rcons: Arc<Mutex<impl Consumer<Item = f32> + Send>>, lprod: Arc<Mutex<impl Producer<Item = f32> + Send>>, rprod: Arc<Mutex<impl Producer<Item = f32> + Send>>, sample_rate: u32, _channels: u16,batch_sz:usize){
        let mut l_biquad_cache: HashMap<(uuid::Uuid, uuid::Uuid), Biquad> = HashMap::new();
        let mut r_biquad_cache: HashMap<(uuid::Uuid, uuid::Uuid), Biquad> = HashMap::new();
        
        loop {
            let mut l_samples = Vec::with_capacity(batch_sz);
            let mut r_samples = Vec::with_capacity(batch_sz);
            let mut l_consumer = lcons.lock().unwrap();
            let mut r_consumer = rcons.lock().unwrap();
            
            for _ in 0..batch_sz {
                if let Some(sample) = l_consumer.try_pop() {
                    l_samples.push(sample);
                }
                if let Some(sample) = r_consumer.try_pop() {
                    r_samples.push(sample);
                }
            }
            
            if l_samples.is_empty() && r_samples.is_empty() {
                std::thread::sleep(std::time::Duration::from_micros(100));
                continue;
            }

            let app_guard = app.lock().unwrap();
            if let Some(profile) = app_guard.profiles.get(app_guard.selected_profile_idx) {
                let l_processed = apply_eq_profile(&l_samples, profile, sample_rate as usize, &mut l_biquad_cache);
                let r_processed = apply_eq_profile(&r_samples, profile, sample_rate as usize, &mut r_biquad_cache);
                drop(app_guard);
                
                let mut l_producer = lprod.lock().unwrap();
                let mut r_producer = rprod.lock().unwrap();
                for (l, r) in l_processed.into_iter().zip(r_processed.into_iter()) {
                    let _ = l_producer.try_push(l);
                    let _ = r_producer.try_push(r);
                }
            }
        }
    }

    fn apply_eq_profile(samples: &Vec<f32>, profile: &EqProfile, sample_rate: usize, biquad_cache: &mut HashMap<(uuid::Uuid, uuid::Uuid), Biquad>) -> Vec<f32> {
        let mut processed = samples.clone();

        if let Some(root) = profile.get_root() {
            let root_node = root.lock().unwrap();
            processed = process_node_recursive(&processed, &root_node, sample_rate, biquad_cache);
        }
        
        if let Some(preamp) = profile.get_preamp() {
            let preamp_gain = 10.0_f32.powf(preamp / 20.0);
            for sample in processed.iter_mut() {
                *sample *= preamp_gain;
            }
        }
        processed
    }

    fn process_node_recursive(samples: &Vec<f32>, node: &EqNode, sample_rate: usize, biquad_cache: &mut HashMap<(Uuid,Uuid), Biquad>) -> Vec<f32> {
        let mut processed = samples.clone();
        for band in &node.bands {
            if band.enabled {
                processed = process_band(&processed, band, node.id, sample_rate, biquad_cache);
            }
        }
        for child in &node.children {
            let child_node = child.lock().unwrap();
            processed = process_node_recursive(&processed, &child_node, sample_rate, biquad_cache);
        }
        processed
    }

    fn process_band(samples: &Vec<f32>, band: &EqBand, node_id: Uuid, sample_rate: usize, biquad_cache: &mut HashMap<(Uuid,Uuid), Biquad>) -> Vec<f32> {
        let cache_key = (node_id, band.id);
        biquad_cache.entry(cache_key).or_insert_with(|| {Biquad::new(band.filter.unwrap_or(EqFilter::Peaking), band.freq, sample_rate, band.q, Some(band.gain as f64))});
        let mut processed = Vec::with_capacity(samples.len());
        for &sample in samples {
            let biquad = biquad_cache.get_mut(&cache_key).unwrap();
            processed.push(biquad.biquad_pass_mut(sample));
        }
        processed
    }
}