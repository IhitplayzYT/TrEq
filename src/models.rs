pub mod models{
    use std::{cell::RefCell, collections::HashMap, f32, rc::Rc, sync::{Arc, Mutex}};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Dao::dao::Dao;


    #[derive(Debug,Serialize,Deserialize)]
    pub struct EqProfile{
        pub id: Uuid,
        pub nodes: HashMap<Uuid,Arc<Mutex<EqNode>>>,
        pub name: String,
        pub preamp: Option<f32>,
        pub root: Option<Arc<Mutex<EqNode>>>,
        pub channel: u8
    }

    impl EqProfile{
        pub fn new(name: &str,preamp: Option<f32>,channel: u8) -> Self{
            Self { id: Uuid::new_v4(), name: name.to_string(), preamp, root: None,nodes: HashMap::new(),channel}
        }

        pub fn add_node(&mut self,parent: Option<Uuid>,node: EqNode){
            let id = node.id.clone();
            let node_rc = Arc::new(Mutex::new(node));
            
            if let Some(paren) = parent{
                if let Some(nd) = self.nodes.get(&paren){
                    nd.lock().unwrap().children.push(Arc::clone(&node_rc));
                }
            } else {
                self.root = Some(Arc::clone(&node_rc));
            }            
            self.nodes.insert(id, node_rc);
        }

        pub fn deactive_node(&mut self,id: Uuid){
            if let Some(x) = self.nodes.get(&id){
                x.lock().unwrap().enabled = false;
            }
        }

        pub fn activate_node(&mut self,id: Uuid){
            if let Some(x) = self.nodes.get(&id){
                x.lock().unwrap().enabled = true;
            }
        }

        pub fn cleanup(&mut self){
            if let Some(z) = &self.root{
                z.lock().unwrap()._inorder(&mut self.nodes,None);
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
                        if let Some(bnd_idx) = nd.lock().unwrap().band_map.get(&bnd_id){
                            use BandUpdate::*;
                            if let Some(op) = o_op{ 
                                match op{
                                    UpdateFilter(fi) => {nd.lock().unwrap().bands[*bnd_idx].update_filter(fi);},
                                    UpdateGain(ga) => {nd.lock().unwrap().bands[*bnd_idx].update_gain(ga);},
                                    UpdateFreq(fr) => {nd.lock().unwrap().bands[*bnd_idx].update_freq(fr);},
                                    UpdateQ(q) => {nd.lock().unwrap().bands[*bnd_idx].update_q(q);},
                                    _ => {}
                                }
                            }
                        }
                    },
                    AddBand => {
                        if let Some(op) = o_op{
                            use BandUpdate::*;
                            match op{
                                Create(fi,ga,fr,sr,q) => {nd.lock().unwrap().add_band(fr,fi, Some(ga),sr,q);},
                                _ => {}
                            }
                        }
                               
                    },
                    DeleteBand(bnd_id) => {
                        nd.lock().unwrap().disable_band(bnd_id);
                    },
                }
            }
        }

        pub fn get_node(&self, id: Uuid) -> Option<Arc<Mutex<EqNode>>> {
            self.nodes.get(&id).cloned()
        }

        pub fn get_root(&self) -> Option<Arc<Mutex<EqNode>>> {
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
        pub children: Vec<Arc<Mutex<EqNode>>>,
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
        
        fn _inorder(&mut self,map: &mut HashMap<Uuid,Arc<Mutex<EqNode>>>,parent: Option<&mut EqNode>){  
            if !self.enabled{
                let children = std::mem::take(&mut self.children);
                for i in children{
                    map.remove(&i.lock().unwrap().id);
                    i.lock().unwrap()._inorder(map,Some(self));
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
                if &v.lock().unwrap().id == id{
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

        pub fn get_child(&self, id: Uuid) -> Option<Arc<Mutex<EqNode>>> {
            for child in &self.children {
                if child.lock().unwrap().id == id {
                    return Some(child.clone());
                }
            }
            None
        }

        pub fn add_child(&mut self, child: Arc<Mutex<EqNode>>) {
            self.children.push(child);
        }

        pub fn remove_child(&mut self, id: Uuid) -> bool {
            if let Some(idx) = self.children.iter().position(|c| c.lock().unwrap().id == id) {
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


    pub struct App{
        pub profiles: Vec<EqProfile>,
        pub dirty: bool,
        pub cur_buff: String,
        pub is_buff_dirty: bool,
        pub dao: Dao,
        pub current_interface: SelectInterfaces,
        pub selected_profile_idx: usize,
        pub selected_node_idx: usize,
        pub selected_band_idx: usize,
        pub selected_filter_idx: usize,
        pub message: Option<String>,
    }

    impl App{
        pub fn new(dir: String) -> Self{
            Self { 
                profiles: vec![], 
                dirty: false, 
                cur_buff:String::new(), 
                is_buff_dirty: false, 
                dao: Dao::new(dir),
                current_interface: SelectInterfaces::default(),
                selected_profile_idx: 0,
                selected_node_idx: 0,
                selected_band_idx: 0,
                selected_filter_idx: 0,
                message: None,
            }
        }

        pub fn is_loaded(&self,name: &str) -> bool{
            self.profiles.iter().any(|x| &x.name[..] == name)
        }


   }


   #[derive(Debug,Clone, Copy,PartialEq, Eq)]
   pub enum SelectInterfaces{
        AddProfile,
        AddNode,
        DeleteNode,
        SelectNode,
        UpdateNode,
        AddEqBand,
        DeleteEqBand,
        SelectEqBand,
        UpdateEqBand,
        SelectFilter,
        ViewEqCurve,
        DeleteProfile,
        SelectProfile,
        SearchProfile,
        UpdateProfile,
        AllProfiles,
        SaveProfile,
        SaveProfiles,
        Finalize,
        Load,
        Unload
   }

   impl Default for SelectInterfaces{
        fn default() -> Self {
            SelectInterfaces::AddProfile
        }

   }

   impl SelectInterfaces{
        pub const ALL: [SelectInterfaces;21] = [SelectInterfaces::AddProfile,SelectInterfaces::AddNode,SelectInterfaces::DeleteNode,SelectInterfaces::SelectNode,SelectInterfaces::UpdateNode,SelectInterfaces::AddEqBand,SelectInterfaces::DeleteEqBand,SelectInterfaces::SelectEqBand,SelectInterfaces::UpdateEqBand,SelectInterfaces::SelectFilter,SelectInterfaces::ViewEqCurve,SelectInterfaces::DeleteProfile,SelectInterfaces::SelectProfile,SelectInterfaces::SearchProfile,SelectInterfaces::UpdateProfile,SelectInterfaces::AllProfiles,SelectInterfaces::SaveProfile,SelectInterfaces::SaveProfiles,SelectInterfaces::Finalize,SelectInterfaces::Load,SelectInterfaces::Unload];

        pub fn name(&self) -> &'static str{
            match self{
                SelectInterfaces::AddProfile => {"AddProfile"},
                SelectInterfaces::AddNode => {"AddNode"},
                SelectInterfaces::DeleteNode => {"DeleteNode"},
                SelectInterfaces::SelectNode => {"SelectNode"},
                SelectInterfaces::UpdateNode => {"UpdateNode"},
                SelectInterfaces::AddEqBand => {"AddEqBand"},
                SelectInterfaces::DeleteEqBand => {"DeleteEqBand"},
                SelectInterfaces::SelectEqBand => {"SelectEqBand"},
                SelectInterfaces::UpdateEqBand => {"UpdateEqBand"},
                SelectInterfaces::SelectFilter => {"SelectFilter"},
                SelectInterfaces::ViewEqCurve => {"ViewEqCurve"},
                SelectInterfaces::DeleteProfile => {"DeleteProfile"},
                SelectInterfaces::SelectProfile => {"SelectProfile"},
                SelectInterfaces::SearchProfile => {"SearchProfile"},
                SelectInterfaces::UpdateProfile => {"UpdateProfile"},
                SelectInterfaces::SaveProfile => {"SaveProfile"},
                SelectInterfaces::SaveProfiles => {"SaveProfiles"},
                SelectInterfaces::Finalize => {"Finalize"},
                SelectInterfaces::Load => {"Load"},
                SelectInterfaces::Unload => {"Unload"},
                SelectInterfaces::AllProfiles => {"AllProfiles"},
            }

        }

        pub fn next(&self) -> SelectInterfaces{
            let idx = Self::ALL.iter().position(|x| x == self).unwrap();
            Self::ALL[(idx+1) % Self::ALL.len()]
        }

        pub fn prev(&self) -> SelectInterfaces{
            let idx = Self::ALL.iter().position(|x| x == self).unwrap();
            Self::ALL[(idx+Self::ALL.len()-1) % Self::ALL.len()]
        }

   }





}