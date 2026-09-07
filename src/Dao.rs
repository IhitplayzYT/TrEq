pub mod dao{
    use std::{error::Error, fs::{self, FileType}, path::PathBuf};

use crate::models::models::EqProfile;


    pub struct Dao{
        pub ip_dir: PathBuf,
        pub all_confs: Vec<String>
        // TODO: Can add a cache for fav EqProfiles
    }

    impl Dao{

        pub fn new(dir: String) -> Self{
            let ip_dir =PathBuf::from(dir);
            if !ip_dir.exists() || ip_dir.is_file(){
                fs::create_dir_all(&ip_dir).unwrap();
            }
            Self { ip_dir: ip_dir,all_confs:vec![]}
        }

        pub fn load_all_confs(&mut self){
            fs::read_dir(&self.ip_dir).unwrap().for_each(|x|{
                let f = x.unwrap();
                if f.file_type().unwrap().is_file() {
                    self.all_confs.push(f.file_name().into_string().unwrap());
                }
            });
        }

        pub fn load_conf(&self,name: &str) -> Result<Option<EqProfile>,Box<dyn Error>>{
            let ret = serde_json::from_str(&(fs::read_to_string(self.ip_dir.join(name))?))?;
            Ok(Some(ret))
        }

        pub fn contains_config(&self,name: &str) -> bool{
            self.ip_dir.join(name).exists()
        }

        pub fn create_config(&self,fname: &str,prof: &EqProfile) -> Result<(),Box<dyn Error>>{
            let json = serde_json::to_string(prof)?;
            fs::write(self.ip_dir.join(fname), json)?;
            Ok(())
        }

        pub fn delete_conf(&self,name: &str) -> Result<(),Box<dyn Error>>{
            fs::remove_file(self.ip_dir.join(name))?;
            Ok(())
        }


    }




}