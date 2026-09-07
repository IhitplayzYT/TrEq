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
                                    UpdateFilter(fi) => {
                                        nd.borrow_mut().bands[*bnd_idx].update_filter(fi);
                                    },
                                    UpdateGain(ga) => {
                                        nd.borrow_mut().bands[*bnd_idx].update_gain(ga);
                                    },
                                    UpdateFreq(fr) => {
                                        nd.borrow_mut().bands[*bnd_idx].update_freq(fr);
                                    },
                                    _ => {}
                                }
                            }
                        }
                    },
                    AddBand => {
                        if let Some(op) = o_op{
                            use BandUpdate::*;
                            match op{
                                UpdateFreq(fr) => {nd.borrow_mut().add_band(fr, None,None);},
                                Create(fi,ga,fr) => {nd.borrow_mut().add_band(fr,fi, Some(ga));},
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
        Create(Option<EqFilter>,f32,Freq)
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

        pub fn add_band(&mut self,freq: Freq,filter: Option<EqFilter>,gain: Option<f32>){
            let band = EqBand::new(freq, filter, gain);
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

    }



    #[derive(Debug,Clone,Copy,Serialize,Deserialize)]
    pub struct EqBand{
        pub id: Uuid,
        pub enabled: bool,
        pub filter: Option<EqFilter>,
        pub freq: Freq,
        pub gain: f32,
    }

    impl EqBand{
        pub fn new(freq: Freq, filter: Option<EqFilter>,gain: Option<f32>) -> Self{
            Self { id: Uuid::new_v4(), enabled: true, filter, freq, gain: gain.unwrap_or(0.0)}
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

    #[derive(Debug,Clone,Copy,Serialize,Deserialize)]
   pub enum Freq{
    Point(usize),
    Range(usize,usize)
   } 





}