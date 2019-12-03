extern crate clap;
extern crate csv;


use clap::{Arg,App, SubCommand};
use std::process::Command;
use std::io::{self, Write};
use std::error::Error;
use std::fmt;
use std::fmt::Debug;

fn main() {
    let clap_matches : clap::ArgMatches = App::new("ZFS Tune Bench")
        .version("0.1")
        .author("Paul Richeson <microuser@scriptedsystems.com")
        .about("Assists a system builder to make decision vdev scructure and ashift parameters by providing benchmark results")
        .arg(Arg::with_name("ashift")
            .short("a")
            .long("ashift")
            .help("speficy the ashift parameter to use on disks when creating zpool")
            .takes_value(true)
        )
        .get_matches();

    let ashift = clap_matches.value_of("ashift").unwrap_or("0");

    let options = ZPoolOptions {
        pool_name : "eight450".to_string(),
        options : vec![],
        devices : vec![
            "/dev/sdk".to_string(),
            "/dev/sdn".to_string(),
            "/dev/sdo".to_string(),
            "/dev/sdp".to_string(),
            "/dev/sdq".to_string(),
            "/dev/sdr".to_string(),
            "/dev/sdj".to_string(),
            "/dev/sds".to_string()
            ],
     }
    ;

    if cfg!(target_os = "windows" ) {
        panic!("windows is not supported");
    }


    println!("Warning this program is really distructive!");
    
    zpool_status(&options);

}
#[derive(Debug)]
struct ValidationError {
    details: String
}
impl ValidationError {
    fn new(msg: &str) -> ValidationError {
        ValidationError{details: msg.to_string()}
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl Error for ValidationError {
    fn description(&self) -> &str {
        &self.details
    }
}

//pub struct Defaults {
//
//    PoolName : String,
//
//}

pub fn zpool_destroy(options : &ZPoolOptions) -> std::process::Output{
    let redirect_out = Command::new("zpool")
        .arg("destroy")
        .arg(&options.pool_name)
        .output()
        .expect(&format!("failed to run zpool destroy {}", &options.pool_name)[..])
        ;
        io::stdout().write_all(&redirect_out.stdout).unwrap();
        io::stderr().write_all(&redirect_out.stderr).unwrap();
    redirect_out
}

pub fn zpool_status(options : &ZPoolOptions) -> std::process::Output{
    let redirect_out = Command::new("zpool")
        .arg("status")
        .arg(&options.pool_name)
        .output()
        .expect(&format!("failed to run zpool status {}", &options.pool_name)[..])
        ;
        io::stdout().write_all(&redirect_out.stdout).unwrap();
        io::stderr().write_all(&redirect_out.stderr).unwrap();
    redirect_out
}


#[derive(Debug,Clone)]
pub struct ZPoolOptions {
    pub pool_name : String,
    pub options : Vec<String>,
    pub devices : Vec<String>,
}

impl Default for ZPoolOptions {
    fn default() -> ZPoolOptions {
        ZPoolOptions {
            pool_name : "".to_string(),
            options : vec![],
            devices : vec![]
        }
    }
}


struct CreatePermutation{
    pub permutations : Vec<Vec<String>>,
}

impl CreatePermutation {
    fn raidz(&mut self, options : &ZPoolOptions){
        //Question: Using powered of copy the pattern of insert, i found that &mut is what I want. Kinda makes since. can explain?
        if options.devices.len() > 1 {
            let mut raid1 = options.devices.clone();
            raid1.insert(0,"raidz1".to_string());
            self.permutations.push(raid1);
        }
        if options.devices.len() > 2 {
            let mut raid2 = options.devices.clone();
            raid2.insert(0,"raid2".to_string());
            self.permutations.push(raid2);
        }
        if options.devices.len() > 3 {
            let mut raid2 = options.devices.clone();
            raid2.insert(0,"raid3".to_string());
            self.permutations.push(raid2);
        }
        ()
    }

    fn mirror(&mut self, options : &ZPoolOptions){
        //TODO: Consider allowing mirrors of 3 drives or more? kinda not needed
        let drive_count = options.devices.len();
        let mut devices_processed = 0;
        if drive_count < 3 {
            return ()
        }
        let mut inner : Vec<String> = vec![];
        for device in options.devices.clone(){
            if devices_processed % 2 == 0 {
                if devices_processed + 1 > drive_count {
                    //for when we have an odd number of drives, the last one can be a spare
                    inner.push("spare".to_string());
                } else {
                    inner.push("mirror".to_string())
                }
            }
            inner.push(device);
            devices_processed += 1;
        }
        self.permutations.push(inner);
        ()

    }

    fn stripe(&mut self, options : &ZPoolOptions){
        let drive_count = options.devices.len();
        if drive_count == 2 {
            let striped2 = options.devices.clone();
            self.permutations.push(striped2);
            return ()
        } 
    
        let mut stripedspare = options.devices.clone();
        if drive_count % 2 == 1 {
            //consider len=3, becomes /dev/sda /dev/sdb spare /dev/sdc
            //consider len=5, becomes /dev/sda /dev/sdb /dev/sdc /dev/sdd spare /dev/sde
            stripedspare.insert(drive_count -1, "spare".to_string());
            self.permutations.push(stripedspare);
        } else {
            self.permutations.push(stripedspare);
        }
        ()
    }
}


impl ZPoolOptions {

    fn get_create_permutations(&self) -> CreatePermutation {
        let mut permutations = CreatePermutation{
            permutations : vec![]
        };
        permutations.mirror(&self);
        permutations.raidz(&self);
        permutations.stripe(&self);
        permutations
    }

    fn is_devices_empty(&self) -> bool{
        self.devices.len() == 0
    }

    fn validate(&self) -> Result< () , Box<ValidationError>> {
        if self.is_valid_pool_name() {
            return Err(From::from(ValidationError::new("Missing Pool Name.")))
        }
        if self.is_devices_empty(){
            return Err(From::from(ValidationError::new("Missing a block device.")))
        }
        match self.is_device_missing(){
            Some(x) => return Err(From::from(ValidationError::new(&format!("Devices do not exist: {}", x.join(","))))),
            None => ()
        };
        
        Ok(())
    }
    fn is_device_missing(&self) -> Option<Vec<String>> {
        let mut missing_devs : Vec<String> = vec![];
        for dev in &self.devices {
            let path = std::path::Path::new(&dev);
            if ! path.exists() {
                missing_devs.push(String::from(path.to_str().unwrap()))
            }
        }
        match missing_devs.is_empty() {
            false => None,
            true => Some(missing_devs)
        }
        
    }


    fn is_valid_pool_name(&self) -> bool {
        self.pool_name.len() == 0
    }

}

pub fn zpool_create(raid_configuration : &Vec<String> , options : &ZPoolOptions) -> std::process::Output {
    //QUESION again here. I needed a clone, I just want to use a read only function
    //let validate = options.clone().validate();
    match options.validate() {
        Err(x) => panic!(x),
        _ => {
            println!("ZPoolOptions validation passed.");
            ()
        },
    }
    println!(
        "Spawning Process: zpool create {} {} {}", 
        &options.options.join(" "), 
        &options.pool_name,
        &options.devices.join(" ")
    );
    let redirect_out : std::process::Output = Command::new("zpool")
        .arg("create")
        .args(&options.options)
        .arg(&options.pool_name)
        .args(raid_configuration)
        .output()
        .expect(&format!("failed to run zpool create {}", &options.pool_name)[..]);
        io::stdout().write_all(&redirect_out.stdout).unwrap();
        io::stderr().write_all(&redirect_out.stderr).unwrap();
    redirect_out
    
        
    
}