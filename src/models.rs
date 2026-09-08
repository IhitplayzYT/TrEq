pub mod models{
    use std::{cell::RefCell, collections::HashMap, f32, rc::Rc};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

    #[derive(Debug,Serialize,Deserialize)]
    pub struct EqProfile{
        id: Uuid,
        nodes: HashMap<Uuid,Rc<RefCell<EqNode>>>,
        name: String,
        preamp: Option<f32>,
        root: Option<Rc<RefCell<EqNode>>>
    }

    impl EqProfile{
        pub fn new(name: &str,preamp: Option<f32>) -> Self{
            Self { id: Uuid::new_v4(), name: name.to_string(), preamp, root: None,nodes: HashMap::new()}
        }

        pub fn add_node(&mut self,parent: Option<Uuid>,node: EqNode){
            let id = node.id.clone();
            let node_rc = Rc::new(RefCell::new(node));
            
            if let Some(paren) = parent{
                if let Some(nd) = self.nodes.get(&paren){
                    nd.borrow_mut().children.push(node_rc.clone());
                }
            } else {
                self.root = Some(node_rc.clone());
            }
            
            self.nodes.insert(id, node_rc);
        }

        pub fn deactive_node(&mut self,id: Uuid){
            if let Some(x) = self.nodes.get(&id){
                x.borrow_mut().enabled = false;
            }
        }

        pub fn activate_node(&mut self,id: Uuid){
            if let Some(x) = self.nodes.get(&id){
                x.borrow_mut().enabled = true;
            }
        }

        pub fn cleanup(&mut self){
            if let Some(z) = &self.root{
                z.borrow_mut()._inorder(&mut self.nodes,None);
            }   
        }

        pub fn finalize(&mut self){
            self.cleanup();
        }

        pub fn update_node(&mut self,nd_id: Uuid,bnd_op: BandOp,o_op: Option<BandUpdate>){
            if let Some(nd) = self.nodes.get(&nd_id){
                use BandOp::*;
                match bnd_op{
                    UpdateBand(bnd_id) => {
                        if let Some(bnd_idx) = nd.borrow().band_map.get(&bnd_id){
                            use BandUpdate::*;
                            if let Some(op) = o_op{ 
                                match op{
                                    UpdateFilter(fi) => {nd.borrow_mut().bands[*bnd_idx].update_filter(fi);},
                                    UpdateGain(ga) => {nd.borrow_mut().bands[*bnd_idx].update_gain(ga);},
                                    UpdateFreq(fr) => {nd.borrow_mut().bands[*bnd_idx].update_freq(fr);},
                                    UpdateQ(q) => {nd.borrow_mut().bands[*bnd_idx].update_q(q);},
                                    _ => {}
                                }
                            }
                        }
                    },
                    AddBand => {
                        if let Some(op) = o_op{
                            use BandUpdate::*;
                            match op{
                                Create(fi,ga,fr,sr,q) => {nd.borrow_mut().add_band(fr,fi, Some(ga),sr,q);},
                                _ => {}
                            }
                        }
                               
                    },
                    DeleteBand(bnd_id) => {
                        nd.borrow_mut().disable_band(bnd_id);
                    },
                }
            }
        }

        pub fn get_node(&self, id: Uuid) -> Option<Rc<RefCell<EqNode>>> {
            self.nodes.get(&id).cloned()
        }

        pub fn get_root(&self) -> Option<Rc<RefCell<EqNode>>> {
            self.root.clone()
        }

        pub fn get_name(&self) -> &str {
            &self.name
        }

        pub fn set_name(&mut self, name: &str) {
            self.name = name.to_string();
        }

        pub fn get_preamp(&self) -> Option<f32> {
            self.preamp
        }

        pub fn set_preamp(&mut self, preamp: Option<f32>) {
            self.preamp = preamp;
        }

        pub fn get_id(&self) -> Uuid {
            self.id
        }

        pub fn node_count(&self) -> usize {
            self.nodes.len()
        }

        pub fn is_empty(&self) -> bool {
            self.nodes.is_empty()
        }

        pub fn clear(&mut self) {
            self.nodes.clear();
            self.root = None;
        }
    }


    #[derive(Debug,Clone, Copy,PartialEq, Eq,Serialize,Deserialize)]
    pub enum BandOp{
        AddBand,
        UpdateBand(Uuid),
        DeleteBand(Uuid)
    }

    #[derive(Debug,Clone, Copy,Serialize,Deserialize)]
    pub enum BandUpdate{
        UpdateFilter(Option<EqFilter>),
        UpdateGain(f32),
        UpdateFreq(Freq),
        UpdateQ(f64),
        Create(Option<EqFilter>,f32,Freq,usize,f64)
    }

    #[derive(Debug,Clone,Serialize,Deserialize)]
    pub struct EqNode{
        pub id: Uuid,
        pub name: Option<String>,
        pub enabled: bool,
        pub bands: Vec<EqBand>,
        pub children: Vec<Rc<RefCell<EqNode>>>,
        pub band_map: HashMap<Uuid,usize>
    }

    impl EqNode{
        pub fn new(name: Option<String>) -> Self{
            Self { id: Uuid::new_v4(), name, enabled: true, bands: vec![], children: vec![],band_map: HashMap::new()}
        }

        pub fn add_band(&mut self,freq: Freq,filter: Option<EqFilter>,gain: Option<f32>,sample_rate:usize,q: f64){
            let band = EqBand::new(freq, filter, gain,sample_rate,q);
            let id = band.id;
            self.bands.push(band);
            self.band_map.insert(id, self.bands.len()-1);
        }
        
        fn _inorder(&mut self,map: &mut HashMap<Uuid,Rc<RefCell<EqNode>>>,parent: Option<&mut EqNode>){  
            if !self.enabled{
                let children = std::mem::take(&mut self.children);
                for i in children{
                    map.remove(&i.borrow().id);
                    i.borrow_mut()._inorder(map,Some(self));
                }
                self.children.clear();
                self.band_map.clear();
                self.bands.clear();
                if let Some(paren) =  parent{
                    paren._delete_child(&self.id);             
                }
            }
        }

        fn _delete_child(&mut self,id: &Uuid){
            for (idx,v) in self.children.iter().enumerate(){
                if &v.borrow().id == id{
                    self.children.remove(idx);
                    break;
                }
            }
        }

        pub fn disable_band(&mut self,id: Uuid){
            if let Some(idx) = self.band_map.get(&id){
                self.bands[*idx].enabled = false;
            }
        }

        pub fn enable_band(&mut self,id: Uuid){
            if let Some(idx) = self.band_map.get(&id){
                self.bands[*idx].enabled = true;
            }
        }

        pub fn get_band(&self, id: Uuid) -> Option<&EqBand> {
            if let Some(idx) = self.band_map.get(&id){
                self.bands.get(*idx)
            } else {
                None
            }
        }

        pub fn get_band_mut(&mut self, id: Uuid) -> Option<&mut EqBand> {
            if let Some(idx) = self.band_map.get(&id){
                self.bands.get_mut(*idx)
            } else {
                None
            }
        }

        pub fn cleanup(&mut self) {
            let mut n_bands= vec![];
            for v in &self.bands{
                if v.enabled{
                    n_bands.push(v.clone());
                }
            }
            self.bands = n_bands;
            self.band_map.clear();
            self.bands.iter().enumerate().for_each(|(idx,x)| {
                self.band_map.insert(x.id, idx);
            });   
        }

        pub fn finalize(&mut self){
            self.cleanup();
        }

        pub fn get_id(&self) -> Uuid {
            self.id
        }

        pub fn get_name(&self) -> Option<&String> {
            self.name.as_ref()
        }

        pub fn set_name(&mut self, name: Option<String>) {
            self.name = name;
        }

        pub fn is_enabled(&self) -> bool {
            self.enabled
        }

        pub fn enable(&mut self) {
            self.enabled = true;
        }

        pub fn disable(&mut self) {
            self.enabled = false;
        }

        pub fn band_count(&self) -> usize {
            self.bands.len()
        }

        pub fn child_count(&self) -> usize {
            self.children.len()
        }

        pub fn get_child(&self, id: Uuid) -> Option<Rc<RefCell<EqNode>>> {
            for child in &self.children {
                if child.borrow().id == id {
                    return Some(child.clone());
                }
            }
            None
        }

        pub fn add_child(&mut self, child: Rc<RefCell<EqNode>>) {
            self.children.push(child);
        }

        pub fn remove_child(&mut self, id: Uuid) -> bool {
            if let Some(idx) = self.children.iter().position(|c| c.borrow().id == id) {
                self.children.remove(idx);
                return true;
            }
            false
        }

        pub fn clear_bands(&mut self) {
            self.bands.clear();
            self.band_map.clear();
        }

        pub fn clear_children(&mut self) {
            self.children.clear();
        }

    }



    #[derive(Debug,Clone,Copy,Serialize,Deserialize)]
    pub struct EqBand{
        pub id: Uuid,
        pub enabled: bool,
        pub filter: Option<EqFilter>,
        pub sample_rate: usize,
        pub q: f64,
        pub freq: Freq,
        pub gain: f32,
    }

    impl EqBand{
        pub fn new(freq: Freq, filter: Option<EqFilter>,gain: Option<f32>,sample_rate: usize,q:f64) -> Self{
            Self { id: Uuid::new_v4(), enabled: true, filter, freq, gain: gain.unwrap_or(0.0),sample_rate,q}
        }

        pub fn update_filter(&mut self,filter: Option<EqFilter>) {
            self.filter = filter
        }

        pub fn update_freq(&mut self,freq: Freq) {
            self.freq = freq;
        }

        pub fn update_gain(&mut self,gain: f32){
            self.gain = gain;
        }

        pub fn update_q(&mut self,q: f64){
            self.q = q;
        }

        pub fn update_sample_rate(&mut self,sample_rate: usize){
            self.sample_rate = sample_rate;
        }

        pub fn get_center_freq(&self) -> usize {
            match self.freq {
                Freq::Point(f) => f,
                Freq::Range(start, end) => (start + end) / 2,
            }
        }

        pub fn get_freq_range(&self) -> Option<(usize, usize)> {
            match self.freq {
                Freq::Range(start, end) => Some((start, end)),
                Freq::Point(_) => None,
            }
        }

        pub fn is_range(&self) -> bool {
            matches!(self.freq, Freq::Range(_, _))
        }

        pub fn is_point(&self) -> bool {
            matches!(self.freq, Freq::Point(_))
        }

        pub fn enable(&mut self){
            self.enabled = true;
        }

        pub fn disable(&mut self){
            self.enabled = false;
        }

    }

    #[derive(Debug,Clone,Copy,Serialize,Deserialize)]
    pub enum EqFilter{
        Peaking,
        LowShelf,
        HighShelf,
        LowPass,
        HighPass,
        BandPass,
        Notch,
    }

    impl ToString for EqFilter{
        fn to_string(&self) -> String {
            use EqFilter::*;
            match self{
                Peaking => {"peaking"},
                LowShelf => {"lowshelf"},
                HighShelf => {"highshelf"},
                LowPass => {"lowpass"},
                HighPass => {"highpass"},
                BandPass => {"bandpass"},
                Notch => {"notch"},               
            }.to_string()
        }
    }

    impl From<String> for EqFilter{
        fn from(value: String) -> Self {
            use EqFilter::*;
            match &value.to_lowercase()[..]{
                "peaking" => {Peaking},
                "lowshelf" => {LowShelf},
                "highshelf" => {HighShelf},
                "lowpass" => {LowPass},
                "highpass" => {HighPass},
                "bandpass" => {BandPass},
                "notch" => {Notch},
                _ => {Peaking}
            }
        }
    }


    #[derive(Debug,Clone,Copy,Serialize,Deserialize)]
   pub enum Freq{
    Point(usize),
    Range(usize,usize)
   } 





}