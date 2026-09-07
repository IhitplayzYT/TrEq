pub mod Helper{
    use std::process::exit;



    const DBG_STR: &str = "";
    const OK:i32 = 0;
    const ERR:i32 = -1;


    #[derive(Debug,Clone)]
    pub struct CLI{
        pub dbg: bool,
        pub profile: Option<String>,
        pub output_profiles_path: String,
    }


    pub fn Help(){
        println!("{DBG_STR}");
        exit(OK);
    }


    impl CLI{
        pub fn new() -> Self{
            Self {dbg: false,profile: None,output_profiles_path: ".Eqfiles/".to_string()}
        }

        pub fn Parse_Args(&mut self){
            let args: Vec<String> = std::env::args().skip(1).collect();
           for i in &args{
                if i == "-d" || i == "--debug" || i == " --DEBUG" || i == "-D"{
                    self.dbg = true;
                } else if i == "-h" || i == "--help" || i == " --HELP" || i == "-H"{
                    Help();
                } else if i.starts_with("--profile=") || i.starts_with("-p="){
                    self.profile = Some(i[i.find("=").unwrap()+1..].to_string());
                } else if i.starts_with("--db_path=") || i.starts_with("-db="){
                    self.output_profiles_path = i[i.find("=").unwrap()+1..].to_string();
                } else{
                    Help();
                }
           } 


        }



    }


    





}