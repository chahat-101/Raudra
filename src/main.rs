use colored::*;
use hdrhistogram::Histogram;
use rand::Rng;
use reqwest::redirect::Policy;
use reqwest::{
    header::{HeaderMap, HeaderValue, FORWARDED, USER_AGENT},
    Client,
};

use std::io;
use std::io::Write;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
use terminal_size::{terminal_size, Width};
use tokio::main;

const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.2.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.5.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.8.0.0.0 Safari/537.36 Edg/100.8.0.0.0",
    "Mozilla/5.0 (Android 11; SM-G960F; rv:120.3.0) Gecko/20100101 Firefox/120.3.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.8.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.1.0.0.0 Safari/537.36 Edg/103.1.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0.0 Safari/537.36",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X; rv:117.6.0) Gecko/20100101 Firefox/117.6.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.3.0.0.0 Safari/537.36 Edg/108.3.0.0.0",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/101.6.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Android 12; Mobile; rv:105.1.0) Gecko/20100101 Firefox/105.1.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/108.3 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/112.6 Safari/605.1.15",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7; rv:91.3.0) Gecko/20100101 Firefox/91.3.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.8.0.0.0 Safari/537.36 Edg/115.8.0.0.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.5.0.0.0 Safari/537.36 Edg/98.5.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/90.2 Safari/605.1.15",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/105.1 Safari/605.1.15",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/90.2 Safari/605.1.15",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/94.3.0.0.0 Safari/537.36 Edg/94.3.0.0.0",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.3.0.0.0 Safari/537.36 Edg/118.3.0.0.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/105.3 Safari/605.1.15",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/101.0.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.2.0.0.0 Safari/537.36 Edg/118.2.0.0.0",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.2.0.0.0 Safari/537.36 Edg/109.2.0.0.0",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/101.9 Safari/605.1.15",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/94.2 Safari/605.1.15",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X; rv:117.1.0) Gecko/20100101 Firefox/117.1.0",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.0.0.0.0 Mobile/Safari/537.36",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.9.0.0.0 Safari/537.36 Edg/104.9.0.0.0",
    "Mozilla/4.0 (compatible; MSIE 10.0.0; X11; Ubuntu; Linux x86_64; Trident/6.0.0)",
    "Mozilla/5.0 (Android 11; SM-G960F; rv:100.2.0) Gecko/20100101 Firefox/100.2.0",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.9.0.0.0 Safari/537.36 Edg/91.9.0.0.0",
    "Mozilla/4.0 (compatible; MSIE 7.0.0; X11; Linux x86_64; Trident/6.0.0)",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.3.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.9.0.0.0 Safari/537.36 Edg/112.9.0.0.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.7.0.0.0 Safari/537.36",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X; rv:95.2.0) Gecko/20100101 Firefox/95.2.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.6.0.0.0 Safari/537.36 Edg/97.6.0.0.0",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.5.0.0.0 Safari/537.36 Edg/120.5.0.0.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/111.2 Safari/605.1.15",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/94.6.0.0.0 Safari/537.36 Edg/94.6.0.0.0",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.5.0.0.0 Mobile/Safari/537.36 Edg/115.5.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3; rv:102.1.0) Gecko/20100101 Firefox/102.1.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.3.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:91.1.0) Gecko/20100101 Firefox/91.1.0",
    "Opera/9.80 (Android 12; Mobile) Presto/2.12.388 Version/11.4",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:95.8.0) Gecko/20100101 Firefox/95.8.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/99.9.0.0.0 Safari/537.36 Edg/99.9.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64; rv:93.3.0) Gecko/20100101 Firefox/93.3.0",
    "Mozilla/5.0 (X11; Linux x86_64; rv:114.7.0) Gecko/20100101 Firefox/114.7.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.6.0.0.0 Mobile/Safari/537.36",
    "Mozilla/5.0 (Android 12; Mobile; rv:101.4.0) Gecko/20100101 Firefox/101.4.0",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.6.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:93.6.0) Gecko/20100101 Firefox/93.6.0",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/100.8 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; WOW64; rv:118.2.0) Gecko/20100101 Firefox/118.2.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.1.0.0.0 Mobile/Safari/537.36 Edg/107.1.0.0.0",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/120.1 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.6.0.0.0 Safari/537.36 Edg/97.6.0.0.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:107.3.0) Gecko/20100101 Firefox/107.3.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:102.9.0) Gecko/20100101 Firefox/102.9.0",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.7.0.0.0 Safari/537.36 Edg/92.7.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.8.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.4.0.0.0 Safari/537.36 Edg/109.4.0.0.0",
    "Mozilla/4.0 (compatible; MSIE 6.0.0; Android 10; Pixel 4 XL; Trident/7.0.0)",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/100.6 Safari/605.1.15",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X; rv:118.6.0) Gecko/20100101 Firefox/118.6.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.7.0.0.0 Safari/537.36 Edg/115.7.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.9.0.0.0 Safari/537.36",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.8.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.2.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.8.0.0.0 Safari/537.36 Edg/111.8.0.0.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.2.0.0.0 Safari/537.36 Edg/107.2.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0.0 Safari/537.36 Edg/120.0.0.0.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.8.0.0.0 Safari/537.36 Edg/96.8.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/107.4 Safari/605.1.15",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.4.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.0.0.0 Mobile/Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.2.0.0.0 Safari/537.36 Edg/112.2.0.0.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.1.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/114.6 Safari/605.1.15",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/112.2 Safari/605.1.15",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.0.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:99.8.0) Gecko/20100101 Firefox/99.8.0",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.5.0.0.0 Safari/537.36 Edg/118.5.0.0.0",
    "Mozilla/5.0 (Android 11; SM-G960F; rv:118.1.0) Gecko/20100101 Firefox/118.1.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/103.6 Safari/605.1.15",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.9.0.0.0 Safari/537.36 Edg/108.9.0.0.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/90.0 Safari/605.1.15",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.8.0.0.0 Safari/537.36 Edg/98.8.0.0.0",
    "Mozilla/4.0 (compatible; MSIE 10.0.0; Macintosh; Intel Mac OS X 10_15_7; Trident/5.0.0)",
    "Opera/9.80 (X11; Linux x86_64) Presto/2.12.388 Version/10.4",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.5.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.1.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64; rv:102.8.0) Gecko/20100101 Firefox/102.8.0",
    "Mozilla/5.0 (Android 12; Mobile; rv:97.7.0) Gecko/20100101 Firefox/97.7.0",
    "Mozilla/5.0 (X11; Linux x86_64; rv:96.2.0) Gecko/20100101 Firefox/96.2.0",
    "Opera/9.80 (X11; Ubuntu; Linux x86_64) Presto/2.12.388 Version/10.8",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.6.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Android 10; Pixel 4 XL; rv:120.0.0) Gecko/20100101 Firefox/120.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/99.1.0.0.0 Safari/537.36 Edg/99.1.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7; rv:108.6.0) Gecko/20100101 Firefox/108.6.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7; rv:99.3.0) Gecko/20100101 Firefox/99.3.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/119.9 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64; rv:107.5.0) Gecko/20100101 Firefox/107.5.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.9.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/96.1 Safari/605.1.15",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X; rv:111.0.0) Gecko/20100101 Firefox/111.0.0",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.2.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.6.0.0.0 Safari/537.36 Edg/119.6.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:118.2.0) Gecko/20100101 Firefox/118.2.0",
    "Opera/9.80 (Android 12; Mobile) Presto/2.12.388 Version/12.4",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/120.2 Safari/605.1.15",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.4.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.4.0.0.0 Safari/537.36 Edg/100.4.0.0.0",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.6.0.0.0 Safari/537.36 Edg/117.6.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.6.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.7.0.0.0 Safari/537.36 Edg/114.7.0.0.0",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.3.0.0.0 Mobile/Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7; rv:107.1.0) Gecko/20100101 Firefox/107.1.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64; rv:99.7.0) Gecko/20100101 Firefox/99.7.0",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.2.0.0.0 Safari/537.36 Edg/90.2.0.0.0",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.8.0.0.0 Safari/537.36 Edg/109.8.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.9.0.0.0 Safari/537.36 Edg/95.9.0.0.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/117.4 Safari/605.1.15",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.1.0.0.0 Safari/537.36 Edg/111.1.0.0.0",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/90.8 Safari/605.1.15",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X; rv:101.0.0) Gecko/20100101 Firefox/101.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.8.0.0.0 Safari/537.36 Edg/118.8.0.0.0",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.0.0.0 Safari/537.36 Edg/98.0.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/101.3.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.0.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.3.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/118.4 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; WOW64; rv:106.0.0) Gecko/20100101 Firefox/106.0.0",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.5.0.0.0 Safari/537.36 Edg/110.5.0.0.0",
    "Opera/9.80 (Macintosh; Intel Mac OS X 10_15_7) Presto/2.12.388 Version/11.8",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.8.0.0.0 Safari/537.36 Edg/102.8.0.0.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.3.0.0.0 Safari/537.36",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X; rv:116.6.0) Gecko/20100101 Firefox/116.6.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL; rv:106.2.0) Gecko/20100101 Firefox/106.2.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7; rv:111.2.0) Gecko/20100101 Firefox/111.2.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:120.2.0) Gecko/20100101 Firefox/120.2.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.4.0.0.0 Safari/537.36 Edg/108.4.0.0.0",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/110.2 Safari/605.1.15",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.8.0.0.0 Safari/537.36 Edg/90.8.0.0.0",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/104.3 Safari/605.1.15",
    "Mozilla/5.0 (Android 11; SM-G960F; rv:99.2.0) Gecko/20100101 Firefox/99.2.0",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.9.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.8.0.0.0 Mobile/Safari/537.36 Edg/91.8.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.9.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:110.8.0) Gecko/20100101 Firefox/110.8.0",
    "Mozilla/5.0 (Android 11; SM-G960F; rv:117.5.0) Gecko/20100101 Firefox/117.5.0",
    "Mozilla/5.0 (X11; Linux x86_64; rv:118.2.0) Gecko/20100101 Firefox/118.2.0",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.9.0.0.0 Safari/537.36",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X; rv:100.2.0) Gecko/20100101 Firefox/100.2.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/116.4 Safari/605.1.15",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.7.0.0.0 Safari/537.36 Edg/112.7.0.0.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.6.0.0.0 Safari/537.36 Edg/116.6.0.0.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/103.8 Safari/605.1.15",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:116.1.0) Gecko/20100101 Firefox/116.1.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.3.0.0.0 Mobile/Safari/537.36 Edg/112.3.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/105.8 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/98.8 Safari/605.1.15",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.8.0.0.0 Safari/537.36",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.5.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/98.7 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/94.5.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3; rv:115.5.0) Gecko/20100101 Firefox/115.5.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:102.4.0) Gecko/20100101 Firefox/102.4.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/99.4 Safari/605.1.15",
    "Mozilla/5.0 (Android 10; Pixel 4 XL; rv:97.1.0) Gecko/20100101 Firefox/97.1.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.9.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3; rv:109.5.0) Gecko/20100101 Firefox/109.5.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/99.6.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:101.7.0) Gecko/20100101 Firefox/101.7.0",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.3.0.0.0 Safari/537.36 Edg/102.3.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64; rv:101.1.0) Gecko/20100101 Firefox/101.1.0",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.2.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.1.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.2.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.6.0.0.0 Safari/537.36 Edg/112.6.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64; rv:116.1.0) Gecko/20100101 Firefox/116.1.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3; rv:110.5.0) Gecko/20100101 Firefox/110.5.0",
    "Mozilla/5.0 (X11; Linux x86_64; rv:91.4.0) Gecko/20100101 Firefox/91.4.0",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.0.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.5.0.0.0 Safari/537.36",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.7.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7; rv:105.3.0) Gecko/20100101 Firefox/105.3.0",
    "Mozilla/5.0 (X11; Linux x86_64; rv:110.2.0) Gecko/20100101 Firefox/110.2.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.1.0.0.0 Safari/537.36 Edg/107.1.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/118.9 Safari/605.1.15",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/91.0 Safari/605.1.15",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X; rv:104.2.0) Gecko/20100101 Firefox/104.2.0",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/104.2 Safari/605.1.15",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/94.7.0.0.0 Safari/537.36 Edg/94.7.0.0.0",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.4.0.0.0 Mobile/Safari/537.36",
    "Mozilla/5.0 (Android 12; Mobile; rv:97.3.0) Gecko/20100101 Firefox/97.3.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.7.0.0.0 Safari/537.36 Edg/113.7.0.0.0",
    "Mozilla/5.0 (Android 12; Mobile; rv:91.2.0) Gecko/20100101 Firefox/91.2.0",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.8.0.0.0 Safari/537.36 Edg/104.8.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/96.7 Safari/605.1.15",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/96.0 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/102.7 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.8.0.0.0 Safari/537.36 Edg/117.8.0.0.0",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.3.0.0.0 Safari/537.36 Edg/90.3.0.0.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/112.0 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/90.7 Safari/605.1.15",
    "Opera/9.80 (Windows NT 6.1; Win64; x64) Presto/2.12.388 Version/11.1",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7; rv:93.7.0) Gecko/20100101 Firefox/93.7.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64; rv:108.1.0) Gecko/20100101 Firefox/108.1.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/113.7 Safari/605.1.15",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/92.3 Safari/605.1.15",
    "Mozilla/5.0 (Android 12; Mobile; rv:91.0.0) Gecko/20100101 Firefox/91.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3; rv:105.5.0) Gecko/20100101 Firefox/105.5.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.5.0.0.0 Safari/537.36 Edg/100.5.0.0.0",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X; rv:116.1.0) Gecko/20100101 Firefox/116.1.0",
    "Mozilla/5.0 (Android 11; SM-G960F; rv:91.9.0) Gecko/20100101 Firefox/91.9.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/111.8 Safari/605.1.15",
    "Mozilla/5.0 (Android 12; Mobile; rv:106.6.0) Gecko/20100101 Firefox/106.6.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64; rv:109.0.0) Gecko/20100101 Firefox/109.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7; rv:94.0.0) Gecko/20100101 Firefox/94.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/106.0 Safari/605.1.15",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/116.3 Safari/605.1.15",
    "Mozilla/5.0 (Android 12; Mobile; rv:116.7.0) Gecko/20100101 Firefox/116.7.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.3.0.0.0 Safari/537.36",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.7.0.0.0 Mobile/Safari/537.36",
    "Mozilla/5.0 (Android 12; Mobile; rv:109.1.0) Gecko/20100101 Firefox/109.1.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X; rv:110.3.0) Gecko/20100101 Firefox/110.3.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL; rv:93.7.0) Gecko/20100101 Firefox/93.7.0",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/91.6 Safari/605.1.15",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.2.0.0.0 Mobile/Safari/537.36 Edg/96.2.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.5.0.0.0 Safari/537.36 Edg/109.5.0.0.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.8.0.0.0 Safari/537.36 Edg/97.8.0.0.0",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.2.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Android 12; Mobile; rv:96.3.0) Gecko/20100101 Firefox/96.3.0",
    "Opera/9.80 (Windows NT 10.0; WOW64) Presto/2.12.388 Version/11.5",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/114.6 Safari/605.1.15",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.6.0.0.0 Safari/537.36 Edg/96.6.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.8.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.8.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.1.0.0.0 Safari/537.36",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/118.6 Safari/605.1.15",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:106.3.0) Gecko/20100101 Firefox/106.3.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/96.8 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.0.0.0 Safari/537.36 Edg/103.0.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.5.0.0.0 Safari/537.36",
    "Mozilla/4.0 (compatible; MSIE 8.0.0; Macintosh; Intel Mac OS X 10_15_7; Trident/4.0.0)",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3; rv:95.0.0) Gecko/20100101 Firefox/95.0.0",
    "Mozilla/4.0 (compatible; MSIE 6.0.0; Windows NT 10.0; Win64; x64; Trident/4.0.0)",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.3.0.0.0 Safari/537.36 Edg/105.3.0.0.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.3.0.0.0 Safari/537.36 Edg/110.3.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.6.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; WOW64; rv:115.6.0) Gecko/20100101 Firefox/115.6.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/110.7 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.6.0.0.0 Safari/537.36 Edg/116.6.0.0.0",
    "Mozilla/5.0 (Android 11; SM-G960F; rv:96.3.0) Gecko/20100101 Firefox/96.3.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.8.0.0.0 Safari/537.36 Edg/97.8.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.3.0.0.0 Safari/537.36 Edg/93.3.0.0.0",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.6.0.0.0 Safari/537.36 Edg/116.6.0.0.0",
    "Mozilla/5.0 (X11; Linux x86_64; rv:103.8.0) Gecko/20100101 Firefox/103.8.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/105.1 Safari/605.1.15",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X; rv:109.6.0) Gecko/20100101 Firefox/109.6.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/113.0 Safari/605.1.15",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X; rv:112.0.0) Gecko/20100101 Firefox/112.0.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/110.7 Safari/605.1.15",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.3.0.0.0 Safari/537.36 Edg/107.3.0.0.0",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.2.0.0.0 Safari/537.36 Edg/92.2.0.0.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/111.9.0.0.0 Safari/537.36 Edg/111.9.0.0.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X; rv:90.0.0) Gecko/20100101 Firefox/90.0.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.2.0.0.0 Mobile/Safari/537.36 Edg/98.2.0.0.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/92.4 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.9.0.0.0 Safari/537.36 Edg/118.9.0.0.0",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.4.0.0.0 Safari/537.36 Edg/110.4.0.0.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/93.4 Safari/605.1.15",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.6.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.6.0.0.0 Safari/537.36 Edg/112.6.0.0.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.6.0.0.0 Safari/537.36 Edg/102.6.0.0.0",
    "Mozilla/5.0 (Android 12; Mobile; rv:110.5.0) Gecko/20100101 Firefox/110.5.0",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.6.0.0.0 Safari/537.36 Edg/114.6.0.0.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.7.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/119.4 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:105.2.0) Gecko/20100101 Firefox/105.2.0",
    "Mozilla/5.0 (Android 11; SM-G960F; rv:120.7.0) Gecko/20100101 Firefox/120.7.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7; rv:115.7.0) Gecko/20100101 Firefox/115.7.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3; rv:92.4.0) Gecko/20100101 Firefox/92.4.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.5.0.0.0 Safari/537.36 Edg/102.5.0.0.0",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/103.6 Safari/605.1.15",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/95.7 Safari/605.1.15",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:106.6.0) Gecko/20100101 Firefox/106.6.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.0.0.0.0 Mobile/Safari/537.36 Edg/93.0.0.0.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X; rv:109.0.0) Gecko/20100101 Firefox/109.0.0",
    "Mozilla/5.0 (X11; Linux x86_64; rv:96.5.0) Gecko/20100101 Firefox/96.5.0",
    "Opera/9.80 (X11; Linux x86_64) Presto/2.12.388 Version/10.3",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.4.0.0.0 Safari/537.36 Edg/102.4.0.0.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.8.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:104.4.0) Gecko/20100101 Firefox/104.4.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.3.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.1.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; WOW64; rv:99.7.0) Gecko/20100101 Firefox/99.7.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64; rv:94.0.0) Gecko/20100101 Firefox/94.0.0",
    "Mozilla/5.0 (Android 11; SM-G960F; rv:111.1.0) Gecko/20100101 Firefox/111.1.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.1.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:118.5.0) Gecko/20100101 Firefox/118.5.0",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/94.3 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.5.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:112.8.0) Gecko/20100101 Firefox/112.8.0",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.7.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.1.0.0.0 Safari/537.36 Edg/109.1.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.4.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/120.3 Safari/605.1.15",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/116.1 Safari/605.1.15",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/112.1 Safari/605.1.15",
    "Mozilla/5.0 (Android 12; Mobile; rv:96.9.0) Gecko/20100101 Firefox/96.9.0",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X; rv:92.6.0) Gecko/20100101 Firefox/92.6.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64; rv:92.2.0) Gecko/20100101 Firefox/92.2.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64; rv:116.1.0) Gecko/20100101 Firefox/116.1.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.7.0.0.0 Safari/537.36 Edg/113.7.0.0.0",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.3.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.9.0.0.0 Safari/537.36 Edg/96.9.0.0.0",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/106.6 Safari/605.1.15",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/108.6 Safari/605.1.15",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/99.5 Safari/605.1.15",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.9.0.0.0 Mobile/Safari/537.36",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64; rv:117.9.0) Gecko/20100101 Firefox/117.9.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/106.8 Safari/605.1.15",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.0.0.0.0 Safari/537.36 Edg/92.0.0.0.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.3.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.7.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/94.4.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; WOW64; rv:101.6.0) Gecko/20100101 Firefox/101.6.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/96.6 Safari/605.1.15",
    "Mozilla/4.0 (compatible; MSIE 8.0.0; Windows NT 10.0; WOW64; Trident/5.0.0)",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.5.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7; rv:98.4.0) Gecko/20100101 Firefox/98.4.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7; rv:104.1.0) Gecko/20100101 Firefox/104.1.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.9.0.0.0 Mobile/Safari/537.36 Edg/114.9.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/113.3 Safari/605.1.15",
    "Mozilla/5.0 (Android 11; SM-G960F; rv:103.8.0) Gecko/20100101 Firefox/103.8.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64; rv:93.8.0) Gecko/20100101 Firefox/93.8.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL; rv:92.7.0) Gecko/20100101 Firefox/92.7.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.5.0.0.0 Safari/537.36 Edg/98.5.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.0.0.0.0 Safari/537.36 Edg/103.0.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0.0 Safari/537.36 Edg/110.0.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.1.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.4.0.0.0 Safari/537.36 Edg/106.4.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7; rv:108.7.0) Gecko/20100101 Firefox/108.7.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0.0 Safari/537.36 Edg/104.0.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/93.6 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; WOW64; rv:117.0.0) Gecko/20100101 Firefox/117.0.0",
    "Mozilla/5.0 (Android 12; Mobile; rv:94.1.0) Gecko/20100101 Firefox/94.1.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64; rv:96.7.0) Gecko/20100101 Firefox/96.7.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.6.0.0.0 Safari/537.36 Edg/96.6.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/109.2 Safari/605.1.15",
    "Mozilla/5.0 (Android 12; Mobile; rv:116.5.0) Gecko/20100101 Firefox/116.5.0",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.8.0.0.0 Mobile/Safari/537.36",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.9.0.0.0 Safari/537.36 Edg/90.9.0.0.0",
    "Mozilla/5.0 (Android 11; SM-G960F; rv:96.2.0) Gecko/20100101 Firefox/96.2.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7; rv:113.7.0) Gecko/20100101 Firefox/113.7.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/101.0 Safari/605.1.15",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.4.0.0.0 Safari/537.36 Edg/95.4.0.0.0",
    "Mozilla/5.0 (Android 12; Mobile; rv:103.7.0) Gecko/20100101 Firefox/103.7.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.5.0.0.0 Safari/537.36 Edg/105.5.0.0.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64; rv:99.0.0) Gecko/20100101 Firefox/99.0.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.0.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/93.3 Safari/605.1.15",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.9.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.3.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64; rv:119.6.0) Gecko/20100101 Firefox/119.6.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64; rv:98.7.0) Gecko/20100101 Firefox/98.7.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL; rv:93.6.0) Gecko/20100101 Firefox/93.6.0",
    "Opera/9.80 (Macintosh; Intel Mac OS X 10_15_7) Presto/2.12.388 Version/11.3",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/113.9 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.2.0) Gecko/20100101 Firefox/109.2.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0.0 Safari/537.36",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.8.0.0.0 Mobile/Safari/537.36 Edg/104.8.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/99.9.0.0.0 Safari/537.36 Edg/99.9.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3; rv:110.1.0) Gecko/20100101 Firefox/110.1.0",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/94.0.0.0.0 Safari/537.36 Edg/94.0.0.0.0",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.3.0.0.0 Mobile/Safari/537.36",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.5.0.0.0 Safari/537.36 Edg/120.5.0.0.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/94.5.0.0.0 Safari/537.36",
    "Opera/9.80 (Windows NT 10.0; Win64; x64) Presto/2.12.388 Version/12.6",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.1.0.0.0 Mobile/Safari/537.36 Edg/96.1.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3; rv:95.6.0) Gecko/20100101 Firefox/95.6.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:96.4.0) Gecko/20100101 Firefox/96.4.0",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X; rv:115.7.0) Gecko/20100101 Firefox/115.7.0",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X; rv:90.2.0) Gecko/20100101 Firefox/90.2.0",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/109.5 Safari/605.1.15",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.2.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/94.7.0.0.0 Safari/537.36 Edg/94.7.0.0.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/118.4 Safari/605.1.15",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.8.0.0.0 Mobile/Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.7.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3; rv:115.7.0) Gecko/20100101 Firefox/115.7.0",
    "Mozilla/5.0 (Android 11; SM-G960F; rv:91.2.0) Gecko/20100101 Firefox/91.2.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7; rv:118.1.0) Gecko/20100101 Firefox/118.1.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/105.2.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/118.0.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.4.0.0.0 Safari/537.36 Edg/90.4.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.9.0.0.0 Safari/537.36 Edg/104.9.0.0.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X; rv:91.4.0) Gecko/20100101 Firefox/91.4.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/101.3.0.0.0 Safari/537.36 Edg/101.3.0.0.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.6.0.0.0 Safari/537.36 Edg/117.6.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/100.9 Safari/605.1.15",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.8.0.0.0 Safari/537.36 Edg/120.8.0.0.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/107.0.0.0.0 Safari/537.36 Edg/107.0.0.0.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/94.6.0.0.0 Safari/537.36 Edg/94.6.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:104.0.0) Gecko/20100101 Firefox/104.0.0",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/109.3 Safari/605.1.15",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/112.6.0.0.0 Safari/537.36 Edg/112.6.0.0.0",
    "Mozilla/4.0 (compatible; MSIE 7.0.0; Macintosh; Intel Mac OS X 10_15_7; Trident/5.0.0)",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/95.2.0.0.0 Safari/537.36 Edg/95.2.0.0.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.2.0.0.0 Safari/537.36",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.3.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64; rv:119.0.0) Gecko/20100101 Firefox/119.0.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X; rv:113.6.0) Gecko/20100101 Firefox/113.6.0",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.9.0.0.0 Safari/537.36 Edg/120.9.0.0.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/112.2 Safari/605.1.15",
    "Opera/9.80 (Windows NT 6.1; Win64; x64) Presto/2.12.388 Version/12.1",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/95.2 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/98.1 Safari/605.1.15",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.8.0.0.0 Safari/537.36 Edg/110.8.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.6.0.0.0 Safari/537.36 Edg/92.6.0.0.0",
    "Mozilla/5.0 (Android 12; Mobile; rv:104.5.0) Gecko/20100101 Firefox/104.5.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:97.5.0) Gecko/20100101 Firefox/97.5.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL; rv:113.7.0) Gecko/20100101 Firefox/113.7.0",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.4.0.0.0 Mobile/Safari/537.36 Edg/100.4.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/96.3 Safari/605.1.15",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3; rv:94.8.0) Gecko/20100101 Firefox/94.8.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:93.6.0) Gecko/20100101 Firefox/93.6.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.2.0.0.0 Safari/537.36 Edg/96.2.0.0.0",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.1.0.0.0 Safari/537.36 Edg/117.1.0.0.0",
    "Mozilla/5.0 (X11; Ubuntu; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/103.4.0.0.0 Safari/537.36 Edg/103.4.0.0.0",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0.0 Safari/537.36 Edg/114.0.0.0.0",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X; rv:93.8.0) Gecko/20100101 Firefox/93.8.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:106.4.0) Gecko/20100101 Firefox/106.4.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.6.0.0.0 Safari/537.36 Edg/113.6.0.0.0",
    "Mozilla/4.0 (compatible; MSIE 11.0.0; Android 12; Mobile; Trident/6.0.0)",
    "Mozilla/4.0 (compatible; MSIE 8.0.0; iPad; CPU OS 14_4 like Mac OS X; Trident/5.0.0)",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/106.6 Safari/605.1.15",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7; rv:117.9.0) Gecko/20100101 Firefox/117.9.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.8.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/91.4 Safari/605.1.15",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X; rv:103.7.0) Gecko/20100101 Firefox/103.7.0",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/92.5.0.0.0 Mobile/Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/104.5 Safari/605.1.15",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.7.0.0.0 Safari/537.36 Edg/109.7.0.0.0",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/94.1.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.7.0.0.0 Safari/537.36 Edg/117.7.0.0.0",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/102.0 Safari/605.1.15",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/108.1 Safari/605.1.15",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.9.0.0.0 Safari/537.36 Edg/106.9.0.0.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/106.8.0.0.0 Safari/537.36 Edg/106.8.0.0.0",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.3.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Android 11; SM-G960F) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/101.7 Safari/605.1.15",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3; rv:118.9.0) Gecko/20100101 Firefox/118.9.0",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/93.1.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/99.8 Safari/605.1.15",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7; rv:98.8.0) Gecko/20100101 Firefox/98.8.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/117.9 Safari/605.1.15",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.5.0.0.0 Safari/537.36",
    "Opera/9.80 (Android 12; Mobile) Presto/2.12.388 Version/11.1",
    "Mozilla/5.0 (Android 12; Mobile; rv:117.5.0) Gecko/20100101 Firefox/117.5.0",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.6.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Android 10; Pixel 4 XL) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/120.2 Safari/605.1.15",
    "Mozilla/5.0 (Android 12; Mobile) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/114.0 Safari/605.1.15",
    "Mozilla/5.0 (X11; Linux x86_64; rv:109.3.0) Gecko/20100101 Firefox/109.3.0",
    "Mozilla/5.0 (Windows NT 10.0; WOW64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.3.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 11_2_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/99.1.0.0.0 Safari/537.36 Edg/99.1.0.0.0",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/117.7.0.0.0 Safari/537.36",
    "Mozilla/5.0 (iPad; CPU OS 14_4 like Mac OS X; rv:100.4.0) Gecko/20100101 Firefox/100.4.0",
    "Mozilla/5.0 (Windows NT 6.1; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/94.0.0.0.0 Safari/537.36 Edg/94.0.0.0.0",
    "Mozilla/5.0 (Android 11; SM-G960F; rv:99.5.0) Gecko/20100101 Firefox/99.5.0",
];

struct Summary {
    total: u32,
    successes: u32,
    failed: u32,
}

impl Summary {
    fn new() -> Self {
        Summary {
            total: 0,
            successes: 0,
            failed: 0,
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
        println!(
            "  {} {}",
            "Total:".bright_white().bold(),
            self.total.to_string().bright_yellow()
        );
        println!(
            "  {} {}",
            "✓ Successes:".bright_green().bold(),
            self.successes.to_string().bright_green()
        );
        println!(
            "  {} {}",
            "✗ Failed:".bright_red().bold(),
            self.failed.to_string().bright_red()
        );

        if self.total > 0 {
            let success_rate = (self.successes as f64 / self.total as f64) * 100.0;
            println!(
                "  {} {}",
                "Success Rate:".bright_white().bold(),
                format!("{success_rate:.2}%").bright_cyan()
            );
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
        println!(
            "{}",
            "⚡ LATENCY ANALYSIS (milliseconds)".bright_magenta().bold()
        );
        println!("{}", "═".repeat(60).bright_magenta());
        println!("  {} {:>10.2} ms", "Minimum:".bright_white(), self.min);
        println!("  {} {:>10.2} ms", "Maximum:".bright_white(), self.max);
        println!("  {} {:>10.2} ms", "Average:".bright_white(), self.mean);
        println!("\n  {}", "Percentiles:".bright_yellow().bold());
        println!(
            "  {} {:>10.2} ms",
            "  P50 (median):".bright_white(),
            self.p50
        );
        println!("  {} {:>10.2} ms", "  P90:".bright_white(), self.p90);
        println!("  {} {:>10.2} ms", "  P99:".bright_white(), self.p99);
        println!("{}", "═".repeat(60).bright_magenta());
    }
}

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print_banner();

    loop {
        println!(
            "\n{}",
            center_text(
                &"╔════════════════════════════════════════════════════════════╗"
                    .bright_blue()
                    .to_string()
            )
        );
        println!(
            "{}",
            center_text(
                &"║           🚀 Ready to start load testing?                 ║"
                    .bright_blue()
                    .to_string()
            )
        );
        println!(
            "{}",
            center_text(
                &"╚════════════════════════════════════════════════════════════╝"
                    .bright_blue()
                    .to_string()
            )
        );
        println!(
            "\n{} {} {} {}",
            "Enter".bright_white(),
            "[Y]".bright_green().bold(),
            "to begin or".bright_white(),
            "[N]".bright_red().bold()
        );
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
                println!(
                    "{}",
                    "❌ Error: Target URL cannot be empty!".bright_red().bold()
                );
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
                    }
                    Ok(_) => {
                        println!(
                            "{}",
                            "❌ Please enter a number greater than 0!".bright_red()
                        );
                        continue;
                    }
                    Err(_) => {
                        println!(
                            "{}",
                            "❌ Invalid input! Please enter a valid number.".bright_red()
                        );
                        continue;
                    }
                }
            };

            // Confirmation
            println!("\n{}", "═".repeat(60).bright_green());
            println!("{}", "📋 Test Configuration".bright_green().bold());
            println!("{}", "═".repeat(60).bright_green());
            println!(
                "  {} {}",
                "Target:".bright_white().bold(),
                target.bright_cyan()
            );
            println!(
                "  {} {}",
                "Requests:".bright_white().bold(),
                requests.to_string().bright_yellow()
            );
            println!("{}", "═".repeat(60).bright_green());

            println!("\n{}", "🔥 Initiating load test...".bright_yellow().bold());
            println!("{}\n", "─".repeat(60).bright_black());

            let mut tasks = vec![];
            let client = Client::builder().redirect(Policy::limited(3)).build()?;

            let resp_summary = Arc::new(Mutex::new(Summary::new()));
            let histogram = Arc::new(Mutex::new(
                Histogram::<u64>::new_with_max(60_000_000, 3).unwrap(),
            ));

            for _i in 0..requests {
                let client = client.clone();
                let target = target.clone();
                let user_agent = user_agent_rotator();
                let fake_ip = random_ip();

                let mut headers = HeaderMap::new();
                headers.insert("X-Forwarded-For", HeaderValue::from_str(&fake_ip)?);
                headers.insert(
                    FORWARDED,
                    HeaderValue::from_str(&format!("for={fake_ip}; proto=https"))?,
                );
                headers.insert(USER_AGENT, HeaderValue::from_str(&user_agent)?);

                let summary = Arc::clone(&resp_summary);
                let histogram = Arc::clone(&histogram);

                tasks.push(tokio::spawn(async move {
                    let start = Instant::now();
                    let resp = client
                        .get(target)
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
                                format!("✓ {status}").bright_green()
                            } else if status.is_client_error() || status.is_server_error() {
                                format!("✗ {status}").bright_red()
                            } else {
                                format!("→ {status}").bright_yellow()
                            };
                            println!("{status_str}");
                            s.add_success();
                        }
                        Err(e) => {
                            println!("{} {:?}", "✗".bright_red(), e);
                            s.add_failed();
                        }
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

    println!(
        "\n{}",
        center_text(&"═".repeat(60).bright_cyan().to_string())
    );
    println!(
        "{}",
        center_text(
            &"👋 Thanks for using Raudra!"
                .bright_cyan()
                .bold()
                .to_string()
        )
    );
    println!("{}", center_text(&"═".repeat(60).bright_cyan().to_string()));
    println!();

    Ok(())
}

fn user_agent_rotator() -> String {
    let mut rng = rand::thread_rng();
    let rand_index = rng.gen_range(0..USER_AGENTS.len());
    USER_AGENTS[rand_index].to_string()
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

        return format!("{a}.{b}.{c}.{d}");
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
            println!(
                "{}",
                center_text(&line.truecolor(255, 140, 0).bold().to_string())
            );
        }
    }

    println!(
        "\n{}",
        center_text(
            &"A blazing-fast HTTP load testing tool"
                .bright_white()
                .italic()
                .to_string()
        )
    );
    println!(
        "{}",
        center_text(&"Built with Rust 🦀".bright_white().italic().to_string())
    );
    println!();
}
