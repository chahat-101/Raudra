use std::io;
use reqwest::redirect::Policy;
use tokio::{main};
use reqwest::{Client,header::{HeaderMap,HeaderValue,FORWARDED,USER_AGENT}};
use std::{sync::{Arc,Mutex}, time::Instant};
use std::fs;
use hdrhistogram::Histogram;    
use rand::Rng; 
use colored::*;

struct Summary{

    total:u32,
    successes:u32,
    failed:u32  
}

impl Summary{
    fn new() ->Self{
        Summary{
            total:0,
            successes:0,
            failed:0
        }
    }

    fn add_success(&mut self) {
        self.successes += 1;
        self.total += 1;

    }

    fn add_failed(&mut self) {
        self.failed += 1;
        self.total += 1;

    }

    fn print_stat(&self) {
        println!("Total: {}| Successes: {}| Failed: {}",self.total,self.successes,self.failed);
    }

}

struct LatencySummary {
    min: f64,
    max: f64,
    mean: f64,
    p50: f64,
    p90: f64,
    p99: f64,
}

impl LatencySummary {
    fn from_histogram(hist: &Histogram<u64>) -> Self {
        Self {
            min: hist.min() as f64 / 1000.0, // Convert microseconds to milliseconds
            max: hist.max() as f64 / 1000.0,
            mean: hist.mean() / 1000.0,
            p50: hist.value_at_quantile(0.50) as f64 / 1000.0,
            p90: hist.value_at_quantile(0.90) as f64 / 1000.0,
            p99: hist.value_at_quantile(0.99) as f64 / 1000.0,
        }
    }

    fn print(&self) {
        println!("\nLatency Summary (ms):");
        println!("  Min:  {:.2}", self.min);
        println!("  Max:  {:.2}", self.max);
        println!("  Mean: {:.2}", self.mean);
        println!("  P50:  {:.2}", self.p50);
        println!("  P90:  {:.2}", self.p90);
        println!("  P99:  {:.2}", self.p99);
    }
}



#[main]
async fn main() -> Result<(), Box<dyn std::error::Error> >{
    print_banner();
    loop {


        let mut input = String::new();
        println!("Do you want to begin!?");
        println!("For yes enter Y/y else N/n");
        
        io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
        
        input = input.trim().to_lowercase();;

        if input == "y"{
            let mut target = String::new();
            println!("Enter the target");
            io::stdin()
            .read_line(&mut target)
            .expect("Failed to read line");
            let requests: u32 = loop {
                let mut input = String::new();

                println!("Enter the number of requests:");

                io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input");


                match input.trim().parse::<u32>() {
                    Ok(num) => break num,
                    Err(_) => {
                        println!("❌ Invalid input! Please enter a valid positive number.\n");
                        continue;
                    }
                }
            };

            let mut tasks = vec![];
            let client = Client::builder()
                    .redirect(Policy::limited(3))
                    .build()?;

            let resp_summary = Arc::new(Mutex::new(Summary::new()));

            let histogram = Arc::new(Mutex::new(
                Histogram::<u64>::new_with_max(60_000_000, 3).unwrap()
            ));


            for _ in 0..requests{
                let client = client.clone();
                let target = target.clone();
                let user_agent = user_agent_rotator()?;
                let fake_ip = random_ip();  

                let mut headers = HeaderMap::new();
                headers.insert(
                    "X-Forwarded-For",
                    HeaderValue::from_str(&fake_ip)?,
                );
                headers.insert(
                    FORWARDED,
                    HeaderValue::from_str(&format!("for={}; proto=https", fake_ip))?,
                );
                headers.insert(
                    USER_AGENT,
                    HeaderValue::from_str(
                        &user_agent,
                    )?,
                );


                let summary = Arc::clone(&resp_summary);
                let histogram = Arc::clone(&histogram);

                tasks.push(tokio::spawn(async move{

                            let start = Instant::now();
                            let resp = client.get(target)
                            .headers(headers)
                            .header("Cache-Control", "no-cache, no-store, must-revalidate")
                            .header("Pragma", "no-cache")
                            .header("Expires", "0")
                            .send()
                            .await;

                            let mut s = summary.lock().unwrap();

                            match &resp {
                                Ok(response) => {
                                    println!("Status: {}", response.status());
                                    s.add_success();
                            },
                                Err(e) => {
                                    println!("Error: {:?}", e);
                                    s.add_failed();
                            },
                            }   

                            let duration = start.elapsed().as_micros() as u64;
                            histogram.lock().unwrap().record(duration).ok();
    
                }));
            }
                for task in tasks{
                    
                    let _ = task.await;

                }      
                
                let summary = resp_summary.lock().unwrap();
                summary.print_stat();
                
                let hist = histogram.lock().unwrap();
                let latency_summary = LatencySummary::from_histogram(&hist);
                
                latency_summary.print();
            
            
            }else{
                break;
            }


            


            
    }
    
    println!("Thanks For Using Raudra");
    Ok(())
        
        
    }

fn user_agent_rotator() -> std::result::Result<String, Box<dyn std::error::Error>> {
    // Read the entire file as a string
    let content = fs::read_to_string("user_agents.txt")?;
    
    // Split by lines and collect into a vector
    let user_agents: Vec<&str> = content.lines()
        .filter(|line| !line.trim().is_empty()) // Filter out empty lines
        .collect();
    
    if user_agents.is_empty() {
        return Err(Box::from("No user agents found in file"));
    }
    
    // Pick a random user agent
    let mut rng = rand::thread_rng();
    let rand_index = rng.gen_range(0..user_agents.len());
    
    Ok(user_agents[rand_index].to_string())
}


fn random_ip() -> String {
    let mut rng = rand::thread_rng();

    loop {
        let a = rng.gen_range(1..=255);
        let b = rng.gen_range(0..=255);
        let c = rng.gen_range(0..=255);
        let d = rng.gen_range(0..=255);

        // Exclude private range 10.0.0.0/8
        if a == 10 {
            continue;
        }

        // Exclude private range 172.16.0.0/12
        if a == 172 && (16..=31).contains(&b) {
            continue;
        }

        // Exclude private range 192.168.0.0/16
        if a == 192 && b == 168 {
            continue;
        }

        // Exclude loopback range 127.0.0.0/8
        if a == 127 {
            continue;
        }

        // Exclude multicast range 224.0.0.0 to 239.255.255.255
        if (224..=239).contains(&a) {
            continue;
        }

        return format!("{}.{}.{}.{}", a, b, c, d);
    }
}

fn print_banner() {
    println!("\n");
    
    
    let logo = r#"
██████╗  █████╗ ██╗   ██╗██████╗ ██████╗  █████╗ 
██╔══██╗██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔══██╗
██████╔╝███████║██║   ██║██║  ██║██████╔╝███████║
██╔══██╗██╔══██║██║   ██║██║  ██║██╔══██╗██╔══██║
██║  ██║██║  ██║╚██████╔╝██████╔╝██║  ██║██║  ██║
╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝
                                                                                                                                                                                                    
"#;
    
    // Print in bright orange color
    for line in logo.lines() {
        println!("{}", line.truecolor(255, 140, 0).bold());
    }
    
    println!("\n");
}



 