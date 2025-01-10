use reqwest::Client;
use std::fs::File;
use std::io::{ copy, Write };
use std::path::Path;
use std::env;
use tokio::fs::create_dir_all;
use indicatif::{ ProgressBar, ProgressStyle };
use futures_util::StreamExt;
use dotenv::dotenv;

async fn download_model_files(
    model_path: &str,
    save_dir: &str
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client.get(model_path).send().await?;

    if response.status().is_success() {
        let file_name = model_path.split('/').last().unwrap_or("model");
        let file_path = Path::new(save_dir).join(file_name);
        let mut file = File::create(file_path)?;

        let total_size = response.content_length().unwrap_or(0);
        let pb = ProgressBar::new(total_size);
        let style = ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})"
            )?
            .progress_chars("#>-");
        pb.set_style(style);

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk)?;
            pb.inc(chunk.len() as u64);
        }

        pb.finish_with_message("Download complete");
        println!("Model downloaded successfully to {}", save_dir);
    } else {
        eprintln!("Failed to download model: {}", response.status());
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    // Parse the argument
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --bin pull <model_path>");
        std::process::exit(1);
    }

    // will update integration of quants more smoother.
    let mut quant = None;
    for i in 1..args.len() {
        if args[i] == "--quant" && i + 1 < args.len() {
            quant = Some(args[i + 1].clone());
        }
    }

    // Define the model path
    let model_path = &args[1];

    // Define the directory to save the model files
    let model_name = model_path.split('/').last().unwrap().split('-').next().unwrap();

    let save_dir: String;
    if let Some(quant_value) = quant {
        save_dir = format!("{}/{}", model_name, quant_value);
        println!("Quant value provided: {}", quant_value);
    } else {
        save_dir = format!("{}", model_name);
        println!("No quant value provided, proceeding without it.");
    }
    let save_dir = env::current_dir()?.join(save_dir);
    let save_dir = save_dir.to_str().unwrap();
    print!("Saving model files to: {}", save_dir);
    create_dir_all(save_dir.clone()).await?;
    // Download the model files
    download_model_files(model_path, &save_dir).await?;
    print!("Model files downloaded successfully.");
    Ok(())
}
