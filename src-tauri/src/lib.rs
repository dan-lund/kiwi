use tract_onnx::prelude::*;
use anyhow::{Result, anyhow};

#[tauri::command]
fn try_load_model(image_bytes: Vec<u8>) -> Result<String, String> {
    match load_model_from_bytes(image_bytes) {
        Ok(class_name) => {
            let out = format!("{}", class_name);
            println!("✅ {}", out);
            Ok(out)
        }
        Err(e) => {
            println!("❌ Error loading model: {:?}", e);
            Err(format!("Failed: {}", e))
        }
    }
}

fn load_model_from_bytes(image_bytes: Vec<u8>) -> Result<String> {
    use std::io::Cursor;

    let model_path = std::path::Path::new("models/model.onnx");

    let labels_path = std::path::Path::new("models/imagenet_slim_labels.txt");

    let model = tract_onnx::onnx()
        .model_for_path(model_path)?
        .into_optimized()?
        .into_runnable()?;

    let reader = Cursor::new(image_bytes);
    let image = image::load(reader, image::ImageFormat::Jpeg)?.to_rgb8();
    let resized = image::imageops::resize(&image, 224, 224, image::imageops::FilterType::Triangle);

    let input: Tensor = tract_ndarray::Array4::from_shape_fn((1, 3, 224, 224), |(_, c, y, x)| {
        let mean = [0.485, 0.456, 0.406][c];
        let std = [0.229, 0.224, 0.225][c];
        (resized[(x as _, y as _)][c] as f32 / 255.0 - mean) / std
    }).into();

    let result = model.run(tvec!(input.into()))?;
    let view = result[0].to_array_view::<f32>()?;

    let class_idx = view
        .iter()
        .cloned()
        .zip(1..) // index starts at 1
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .map(|(_, idx)| idx)
        .ok_or_else(|| anyhow!("Model output was empty"))?;

    let labels: Vec<String> = std::fs::read_to_string(labels_path)?
        .lines()
        .map(|l| l.to_owned())
        .collect();

    let class_name = labels
        .get(class_idx)
        .ok_or_else(|| anyhow!("No label for index {}", class_idx))?
        .clone();

    Ok(class_name)
    
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            try_load_model 
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}