use reqwest::Client;
use std::fs::File;
use std::io::copy;
use std::path::Path;
use tokio::fs::create_dir_all;

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
        let mut content = response.bytes().await?;
        copy(&mut content.as_ref(), &mut file)?;
        println!("Model downloaded successfully to {}", save_dir);
    } else {
        eprintln!("Failed to download model: {}", response.status());
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse the argument
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run --example pull <model_path>");
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
        save_dir = format!("./models/{}/{}", model_name, quant_value);
        println!("Quant value provided: {}", quant_value);
    } else {
        save_dir = format!("./models/{}", model_name);
        println!("No quant value provided, proceeding without it.");
    }

    print!("Saving model files to: {}", save_dir);
    create_dir_all(save_dir.clone()).await?;
    // Download the model files
    download_model_files(model_path, &save_dir).await?;
    print!("Model files downloaded successfully.");
    Ok(())
}
