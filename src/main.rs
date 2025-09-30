use clap::Parser;
use hdrhistogram::Histogram;
use reqwest::{Client,Method};
use serde::Serialize;
use std::collections::HashMap;
use std::string;
use std::sync::{Arc,Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;

#[derive(Parser,Debug)]
#[command(author,version,about = "Website load handling checker with global RPS (Requests per second)")]
struct Args{

    // Target URLS (Round-Robins)
    #[arg(short = 'u',long,required = true)]
    urls: Vec<String>,

    #[arg(short = 'm',long,default_value = "GET")]
    method:String,

    // Headers as Key:Value pairs
    #[arg(short = 'H',long = "header")]
    headers:Vec<String>,

    // Request Body, this is required for Post/Put requests.
    #[arg(short = 'H',long)]
    body:Option<String>,

    #[arg(short = 'c',long,default_value_t = 50)]
    concurrency:usize,

    // Global Request Per Second (by default it will be 0, so no limit)
    #[arg(short = 'r',long,default_value_t = 0)]
    rps:u64,

    #[arg(short = 'd',long)]
    duration_sec:Option<u64>,

    #[arg(short = 'n',long)]
    requests:Option<u64>,

    #[arg(short = 't',long)]
    timeout_ms:u64,
    
    #[arg(short = 'o',long)]
    out:Option<String>,

    #[arg(long,default_value_t = true)]
    print:bool


}

#[derive(Serialize,Debug)]
struct Summary{
    runtime_s:f64,
    total_requests:u64,
    successes:u64,
    failures:u64,
    achieved_ms:LatencySummary
}

#[derive(Serialize,Debug)]
struct LatencySummary{
    count:u64,
    min_ms:f64,
    max_ms:f64,
    mean_ms:f64,
    p50_ms:f64,
    p90_ms:f64,
    p99_ms:f64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{

    let args = Args::parse();

    // Basic Validation: either duration or number of requests must be set
    if args.duration_sec.is_none() && args.requests.is_none() {
        anyhow::bail!("Either --duration--ser or --requests must be provided");
    }
    if args.duration_sec.is_some() && args.requests.is_some() {
        anyhow::bail!("Provide only --duration--sec or --requests");
    }

    // Defining the Method (Get/Put/etc)
    let method = args.method.parse::<Method>().unwrap_or(Method::GET);

    
    Ok(())

}