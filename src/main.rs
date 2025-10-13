// use anyhow::Ok;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use clap::Parser;
use hdrhistogram::Histogram;
use reqwest::{Client,Method,};
use serde::Serialize;
use std::sync::{Arc,Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc,Mutex as TokioMutex};
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
    #[arg(short = 'h',long = "header")]
    headers:Vec<String>,

    // Request Body, this is required for Post/Put requests.
    #[arg(short = 'b',long)]
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
    achieved_rps: f64,
    latency_ms:LatencySummary
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

#[derive(Default)]
struct Metrics {
    total: AtomicU64,
    success: AtomicU64,
    failed: AtomicU64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()>{

    let args = Args::parse();

    // Basic Validation: either duration or number of requests must be set
    if args.duration_sec.is_none() && args.requests.is_none() {
        anyhow::bail!("Either --duration--sec or --requests must be provided");
    }
    if args.duration_sec.is_some() && args.requests.is_some() {
        anyhow::bail!("Provide only --duration--sec or --requests");
    }

    // Defining the Method (Get/Put/etc)
    let method = args.method.parse::<Method>().unwrap_or(Method::GET);

    let mut header_map = reqwest::header::HeaderMap::new();

    for h in &args.headers {

        if let Some(idx) = h.find(":"){
            let key = h[..idx].trim();
            let val = h[idx+1..].trim();

            
            if let Result::Ok(name) = reqwest::header::HeaderName::from_bytes(key.as_bytes()) {
                if let Result::Ok(value) = reqwest::header::HeaderValue::from_str(val) {
                    header_map.insert(name, value);
                }
            }
        }

    }

    // Building Client
    let client = Client::builder()
                                .redirect(reqwest::redirect::Policy::limited(10))
                                .build()?;

    
    let metrics = Arc::new(Metrics::default());
    let stop_flag = Arc::new(AtomicBool::new(false));
    let histograms: Arc<Mutex<Vec<Histogram<u64>>>> = Arc::new(Mutex::new(Vec::new()));


    let requests_budget = Arc::new(AtomicU64::new(args.requests.unwrap_or(0)));

    let token_reciever_opt:Option<Arc<TokioMutex<mpsc::Receiver<()>>>> = if args.rps == 0{
        None
    } 
    else{
        let cap = (args.rps*2).max(100) as usize;
        let (tx,rx)= mpsc::channel::<()>(cap);
        let tx_clone = tx.clone();
        let rps = args.rps;
        tokio::spawn(async move{
            let tick_ms = 100u64;
            let per_tick = (rps as f64 * (tick_ms as f64 /1000.0)).round() as usize;
            let mut interval = tokio::time::interval(Duration::from_millis(tick_ms));
            
            loop {
                interval.tick().await;
                
                for _ in 0..per_tick{
                    if tx_clone.send(()).await.is_err(){
                        return ;
                    }
                }
            }
        
        });
        Some(Arc::new(TokioMutex::new(rx)))
    };

    

    let start = Instant::now();
    let mut handles = Vec::with_capacity(args.concurrency);
    for worker_id in 0..args.concurrency{
        
        let client = client.clone();
        let urls = args.urls.clone();
        
        let metrices = metrics.clone();
        let stop_flag = Arc::clone(&stop_flag);
        
        let histograms = Arc::clone(&histograms);
        let requests_budget = Arc::clone(&requests_budget);
        
        let token_reciever_clone = token_reciever_opt.as_ref().map(Arc::clone);
        let headers = header_map.clone();
        
        let body = args.body.clone();
        let timeout = Duration::from_millis(args.timeout_ms);
        
        let method = method.clone();
        let rps = args.rps;
        {
        let mut guard = histograms.lock().unwrap();
        guard.push(Histogram::<u64>::new_with_max(10_000_000,3).unwrap());
        }

        let handle = tokio::spawn(async move {
            let mut local_count: u64 = 0;
            let try_reserve = |budget:&AtomicU64| -> bool{
                let mut cur:u64 = budget.load(Ordering::SeqCst);
                loop{
                    if cur == 0{
                        return false;
                    }
                    match budget.compare_exchange(cur, cur - 1, Ordering::SeqCst, Ordering::SeqCst){
                        Ok(_) => return true,
                        Err(now) => cur = now,
                    }
                }
            };
            
            loop {
                if stop_flag.load(Ordering::Relaxed){
                    break;
                }

                if args.requests.is_some(){
                    if !try_reserve(&requests_budget){
                        break;
                    }
                }
                if rps>0{
                    if let Some(rx_arc) = &token_reciever_clone{
                        let mut rx = rx_arc.lock().await;
                        if rx.recv().await.is_none() {
                            break;
                        }
                    }
                }

                let idx = (local_count as usize) % urls.len();
                local_count = local_count.wrapping_add(1);
                let url = &urls[idx];

                let req_start = Instant::now();

                let mut req_builder = client.request(method.clone(), url).headers(headers.clone()).timeout(timeout);
                
                if let Some(ref b) = body{
                    req_builder = req_builder.body(b.clone());
                }
                
                let resp = req_builder.send().await;
                let elapsed = req_start.elapsed();
                let micros = elapsed.as_micros() as u64;

                metrices.total.fetch_add(1, Ordering::Relaxed);
                match resp{
                    Ok(resp) => {
                        if resp.status().is_success(){
                            metrices.success.fetch_add(1, Ordering::Relaxed);
                        } else{
                            metrices.failed.fetch_add(1, Ordering::Relaxed);
                        }
                    }

                    Err(_) => {
                        metrices.failed.fetch_add(1, Ordering::Relaxed);
                    }
                }

                {
                    let mut guard = histograms.lock().unwrap();
                    if let Some(h) = guard.get_mut(worker_id){
                        let v = if micros == 0{1}else{micros};
                        let _ = h.record(v);
                    }
                }
            }
        });

        handles.push(handle);
    }

    if let Some(d) = args.duration_sec{
        sleep(Duration::from_secs(d)).await;
        stop_flag.store(true, Ordering::Relaxed);
   } else if args.requests.is_some() {
        while requests_budget.load(Ordering::SeqCst) > 0{
            if stop_flag.load(Ordering::Relaxed){
                break;
            }
            sleep(Duration::from_millis(50)).await;

        }
        stop_flag.store(true,Ordering::Relaxed);
   }

   sleep(Duration::from_millis(200)).await;
   for h in handles{
    let _ = h.await;
   }

   let elapsed = start.elapsed();
   let total = metrics.total.load(Ordering::Relaxed);
   let success = metrics.success.load(Ordering::Relaxed);
   let failed = metrics.failed.load(Ordering::Relaxed);
   let elapsed_s = elapsed.as_secs_f64();
   let achieved_rps = if elapsed_s > 0.0 { total as f64/elapsed_s } else{0.0};


   let merged = {
    let guard = histograms.lock().unwrap();
    let mut merged = Histogram::<u64>::new_with_max(10_000_000, 3).unwrap();
    for h in guard.iter(){
        let _ = merged.add(h);
    }
    merged
   };

   let count = merged.len();
   let (min,max,mean,p50,p90,p99) = if count>0{
    (
        merged.min() as f64/ 1000.0,
        merged.max() as f64/ 1000.0,
        merged.mean()/1000.0,
        merged.value_at_quantile(0.5) as f64/1000.0,
        merged.value_at_quantile(0.9) as f64/1000.0,
        merged.value_at_quantile(0.99) as f64/1000.0,
    )
   }else{
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
   };

   let summary = Summary{
        runtime_s: elapsed_s,
        total_requests:total,
        successes:success,
        failures:failed,
        achieved_rps:achieved_rps,
        latency_ms:LatencySummary{
            count,
            min_ms:min,
            max_ms:max,
            mean_ms:mean,
            p50_ms:p50,
            p90_ms:p90,
            p99_ms:p99
        },
   };
    

    if args.print{
        println!("\n==== Load Handling Summary ====");
        println!("Runtime: {:.3} s", summary.runtime_s);
        println!("Total requests: {}", summary.total_requests);
        println!("  Successes: {}", summary.successes);
        println!("  Failures:  {}", summary.failures);
        println!("Achieved RPS: {:.2}", summary.achieved_rps);
        println!("Latency (ms) count = {}", summary.latency_ms.count);
    }
    if summary.latency_ms.count > 0 {
            println!("  min:  {:.3}", summary.latency_ms.min_ms);
            println!("  mean: {:.3}", summary.latency_ms.mean_ms);
            println!("  max:  {:.3}", summary.latency_ms.max_ms);
            println!("  p50:  {:.3}", summary.latency_ms.p50_ms);
            println!("  p90:  {:.3}", summary.latency_ms.p90_ms);
            println!("  p99:  {:.3}", summary.latency_ms.p99_ms);
        }
    
    if let Some(path) = args.out {
        let j = serde_json::to_string_pretty(&summary)?;
        tokio::fs::write(path, j).await?;
    }
    
    Ok(())

}   