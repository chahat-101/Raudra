use clap::Parser;
use serde::{Serialize};
use tokio::{main,sync::Mutex as Mutex};
use reqwest::{Client,Request,header};
use std::sync::{Arc};
#[derive(Parser,Debug)]
struct Args{

    // number of concurrent requests that the user wants to make
    #[arg(short,long)]
    requests:u32,
    #[arg(short,long)]
    duration:u32,

    // url and ip are mutually exclisive
    #[arg(short,long,default_value = "NONE")]
    url:String,
    #[arg(short,long,default_value = "NONE")]
    ip:String,
}

#[derive(Serialize,Debug)]
struct Summary{

    total:u32,
    successes:u32,
    failed:u32
}

#[derive(Serialize)]
struct Latency_Summary {
    min: f32,
    max: f32,
    mean: f32,
    p50: f32,
    p90: f32,
    p99: f32,
}

struct UserAgentRotator{
    agents:Vec<String>,
    counter:Arc<Mutex<usize>>
}

impl UserAgentRotator{
    fn new(agent:Vec<String>) -> Self{
        Self{
            agents:agent,
            counter:Arc::new(Mutex::new(0))
        }
    }

    async fn get_next(&self) -> String{
        let mut counter = self.counter.lock().await;
        let agent = self.agents[*counter%self.agents.len()].clone();
        *counter +=1;
        agent
    }
}


#[main]
async fn main() -> Result<(), Box<dyn std::error::Error> >{

    let args = Args::parse();
    let client = Client::new();
    let target = if args.url != "NONE" && args.ip == "NONE"{
        args.url.clone()
    } else if args.url == "NONE" && args.ip != "NONE"{
        args.ip.clone()
    } else if args.url != "NONE" && args.ip != "NONE"{
        eprintln!("Error: Cannot use both url and ip. Please provide only one.");
        return Ok(())
    } else{
        eprintln!("Error: Please provide either --url or --ip");
        return Ok(())
    };


    println!("=== Load Test Configuration ===");
    println!("Target: {}", target);
    println!("Concurrent Requests: {}", args.requests);
    println!("Duration: {}s", args.duration);
    println!("================================\n");
    let tot_reqs = args.requests;
    let handles:Vec<_> = (0..tot_reqs).map(|i|{
        let client = Client::new();
        let target = target.to_string();
        tokio::spawn(async move {
            client.get(&target).header(header::CONNECTION, "close").send();
        })
        }  
    ).collect();

    let mut summary =  Summary{
        total:0,
        successes:0,
        failed:0,
    };
    for handle in handles{
        match handle.await{
            Ok(_)=>{
                summary.successes += 1;
                summary.total += 1;
            }
            Err(_) => {
                summary.failed += 1;
                summary.total += 1;

            }
        }


        println!("{:?}",summary);



    }
        Ok(())
    


}





 