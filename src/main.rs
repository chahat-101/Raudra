use std::io;
use std::io::Write;
use reqwest::redirect::Policy;
use tokio::{main};
use reqwest::{Client,header::{HeaderMap,HeaderValue,FORWARDED,USER_AGENT}};
use std::{sync::{Arc,Mutex}, time::Instant};
use std::fs;
use hdrhistogram::Histogram;    
use rand::Rng; 
use colored::*;
use terminal_size::{Width, terminal_size};

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
        println!("\n{}", "═".repeat(60).bright_cyan());
        println!("{}", "📊 REQUEST SUMMARY".bright_cyan().bold());
        println!("{}", "═".repeat(60).bright_cyan());
        println!("  {} {}", "Total:".bright_white().bold(), self.total.to_string().bright_yellow());
        println!("  {} {}", "✓ Successes:".bright_green().bold(), self.successes.to_string().bright_green());
        println!("  {} {}", "✗ Failed:".bright_red().bold(), self.failed.to_string().bright_red());
        
        if self.total > 0 {
            let success_rate = (self.successes as f64 / self.total as f64) * 100.0;
            println!("  {} {}", "Success Rate:".bright_white().bold(), 
                     format!("{:.2}%", success_rate).bright_cyan());
        }
        println!("{}", "═".repeat(60).bright_cyan());
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
            min: hist.min() as f64 / 1000.0,
            max: hist.max() as f64 / 1000.0,
            mean: hist.mean() / 1000.0,
            p50: hist.value_at_quantile(0.50) as f64 / 1000.0,
            p90: hist.value_at_quantile(0.90) as f64 / 1000.0,
            p99: hist.value_at_quantile(0.99) as f64 / 1000.0,
        }
    }

    fn print(&self) {
        println!("\n{}", "═".repeat(60).bright_magenta());
        println!("{}", "⚡ LATENCY ANALYSIS (milliseconds)".bright_magenta().bold());
        println!("{}", "═".repeat(60).bright_magenta());
        println!("  {} {:>10.2} ms", "Minimum:".bright_white(), self.min);
        println!("  {} {:>10.2} ms", "Maximum:".bright_white(), self.max);
        println!("  {} {:>10.2} ms", "Average:".bright_white(), self.mean);
        println!("\n  {}", "Percentiles:".bright_yellow().bold());
        println!("  {} {:>10.2} ms", "  P50 (median):".bright_white(), self.p50);
        println!("  {} {:>10.2} ms", "  P90:".bright_white(), self.p90);
        println!("  {} {:>10.2} ms", "  P99:".bright_white(), self.p99);
        println!("{}", "═".repeat(60).bright_magenta());
    }
}

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error> >{
    print_banner();
    
    loop {
        println!("\n{}", center_text(&"╔════════════════════════════════════════════════════════════╗".bright_blue().to_string()));
        println!("{}", center_text(&"║           🚀 Ready to start load testing?                 ║".bright_blue().to_string()));
        println!("{}", center_text(&"╚════════════════════════════════════════════════════════════╝".bright_blue().to_string()));
        println!("\n{} {} {} {}", 
                 "Enter".bright_white(), 
                 "[Y]".bright_green().bold(), 
                 "to begin or".bright_white(),
                 "[N]".bright_red().bold());
        print!("{} ", "→".bright_cyan().bold());
        io::stdout().flush().ok();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        
        input = input.trim().to_lowercase();

        if input == "y" || input == "yes" {
            // Get target URL
            println!("\n{}", "─".repeat(60).bright_black());
            println!("{}", "🎯 Target Configuration".bright_yellow().bold());
            println!("{}", "─".repeat(60).bright_black());
            print!("{} ", "Enter target URL:".bright_white());
            io::stdout().flush().ok();
            
            let mut target = String::new();
            io::stdin()
                .read_line(&mut target)
                .expect("Failed to read line");
            target = target.trim().to_string();

            if target.is_empty() {
                println!("{}", "❌ Error: Target URL cannot be empty!".bright_red().bold());
                continue;
            }

            // Get number of requests
            let requests: u32 = loop {
                println!("\n{}", "─".repeat(60).bright_black());
                print!("{} ", "Enter number of requests:".bright_white());
                io::stdout().flush().ok();

                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read input");

                match input.trim().parse::<u32>() {
                    Ok(num) if num > 0 => {
                        break num;
                    },
                    Ok(_) => {
                        println!("{}", "❌ Please enter a number greater than 0!".bright_red());
                        continue;
                    },
                    Err(_) => {
                        println!("{}", "❌ Invalid input! Please enter a valid number.".bright_red());
                        continue;
                    }
                }
            };

            // Confirmation
            println!("\n{}", "═".repeat(60).bright_green());
            println!("{}", "📋 Test Configuration".bright_green().bold());
            println!("{}", "═".repeat(60).bright_green());
            println!("  {} {}", "Target:".bright_white().bold(), target.bright_cyan());
            println!("  {} {}", "Requests:".bright_white().bold(), requests.to_string().bright_yellow());
            println!("{}", "═".repeat(60).bright_green());
            
            println!("\n{}", "🔥 Initiating load test...".bright_yellow().bold());
            println!("{}\n", "─".repeat(60).bright_black());

            let mut tasks = vec![];
            let client = Client::builder()
                    .redirect(Policy::limited(3))
                    .build()?;

            let resp_summary = Arc::new(Mutex::new(Summary::new()));
            let histogram = Arc::new(Mutex::new(
                Histogram::<u64>::new_with_max(60_000_000, 3).unwrap()
            ));

            for _i in 0..requests {
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
                    HeaderValue::from_str(&user_agent)?,
                );

                let summary = Arc::clone(&resp_summary);
                let histogram = Arc::clone(&histogram);

                tasks.push(tokio::spawn(async move {
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
                            let status = response.status();
                            let status_str = if status.is_success() {
                                format!("✓ {}", status).bright_green()
                            } else if status.is_client_error() || status.is_server_error() {
                                format!("✗ {}", status).bright_red()
                            } else {
                                format!("→ {}", status).bright_yellow()
                            };
                            println!("{}", status_str);
                            s.add_success();
                        },
                        Err(e) => {
                            println!("{} {:?}", "✗".bright_red(), e);
                            s.add_failed();
                        },
                    }

                    let duration = start.elapsed().as_micros() as u64;
                    histogram.lock().unwrap().record(duration).ok();
                }));
            }

            for task in tasks {
                let _ = task.await;
            }

            let summary = resp_summary.lock().unwrap();
            summary.print_stat();
            
            let hist = histogram.lock().unwrap();
            let latency_summary = LatencySummary::from_histogram(&hist);
            latency_summary.print();

        } else if input == "n" || input == "no" {
            break;
        } else {
            println!("{}", "❌ Invalid input! Please enter Y or N.".bright_red());
        }
    }
    
    println!("\n{}", center_text(&"═".repeat(60).bright_cyan().to_string()));
    println!("{}", center_text(&"👋 Thanks for using Raudra!".bright_cyan().bold().to_string()));
    println!("{}", center_text(&"═".repeat(60).bright_cyan().to_string()));
    println!();
    
    Ok(())
}

fn user_agent_rotator() -> std::result::Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string("user_agents.txt")?;
    
    let user_agents: Vec<&str> = content.lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    
    if user_agents.is_empty() {
        return Err(Box::from("No user agents found in file"));
    }
    
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

        if a == 10 {
            continue;
        }

        if a == 172 && (16..=31).contains(&b) {
            continue;
        }

        if a == 192 && b == 168 {
            continue;
        }

        if a == 127 {
            continue;
        }

        if (224..=239).contains(&a) {
            continue;
        }

        return format!("{}.{}.{}.{}", a, b, c, d);
    }
}

fn get_terminal_width() -> usize {
    if let Some((Width(w), _)) = terminal_size() {
        w as usize
    } else {
        80 // Default fallback
    }
}

fn center_text(text: &str) -> String {
    let width = get_terminal_width();
    let text_len = text.chars().count();
    if text_len >= width {
        return text.to_string();
    }
    let padding = (width - text_len) / 2;
    format!("{}{}", " ".repeat(padding), text)
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
    
    for line in logo.lines() {
        if !line.trim().is_empty() {
            println!("{}", center_text(&line.truecolor(255, 140, 0).bold().to_string()));
        }
    }
    
    println!("\n{}", center_text(&"A blazing-fast HTTP load testing tool".bright_white().italic().to_string()));
    println!("{}", center_text(&"Built with Rust 🦀".bright_white().italic().to_string()));
    println!();
}