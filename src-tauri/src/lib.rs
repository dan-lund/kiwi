use tract_onnx::prelude::*;
use anyhow::{Result};
use std::{path::Path, sync::{Arc, Mutex}};
use std::io::Cursor;
use tauri::State;
use tauri::async_runtime::spawn_blocking;

type TypedRunnableModel = RunnableModel<TypedFact, Box<dyn TypedOp>, TypedModel>;

struct ModelState(Arc<Mutex<TypedRunnableModel>>);

#[tauri::command]
async fn run_inference(state: State<'_, ModelState>, image_bytes: Vec<u8>) -> Result<String, String> {
    
    // Extract an owned Arc<Mutex<…>> out of the State
    let model_arc: Arc<Mutex<TypedRunnableModel>> = state.0.clone();

    // Spawn the heavy work onto a blocking thread
    let class_name = spawn_blocking(move || {

        // Lock the model:
        let model = model_arc
            .lock()
            .map_err(|_| "Model mutex was poisoned".to_string())?;

        // Preprocess 
        let reader = Cursor::new(image_bytes);
        let image = image::load(reader, image::ImageFormat::Jpeg)
            .map_err(|e| format!("Image decode error: {}", e))?
            .to_rgb8();
        let resized = image::imageops::resize(
            &image,
            224, 224,
            image::imageops::FilterType::Triangle,
        );
        let input: Tensor = tract_ndarray::Array4::from_shape_fn((1,3,224,224), |(_, c, y, x)| {
            let mean = [0.485,0.456,0.406][c];
            let std  = [0.229,0.224,0.225][c];
            (resized[(x as _, y as _)][c] as f32 / 255.0 - mean) / std
        }).into();

        // Run inference
        let outputs = model
            .run(tvec!(input.into()))
            .map_err(|e| format!("Inference error: {}", e))?;
        let view = outputs[0]
            .to_array_view::<f32>()
            .map_err(|e| format!("Output view error: {}", e))?;

        let best_idx = view
            .iter()
            .cloned()
            .zip(1..)
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .map(|(_, idx)| idx)
            .ok_or_else(|| "Empty output".to_string())?;

        // Read labels 
        let labels: Vec<String> = std::fs::read_to_string("models/imagenet_slim_labels.txt")
            .map_err(|e| format!("Labels load failed: {}", e))?
            .lines()
            .map(|l| l.to_owned())
            .collect();

        labels
            .get(best_idx)
            .cloned()
            .ok_or_else(|| format!("No label at index {}", best_idx))
    })
    .await
    .map_err(|e| format!("Join error: {}", e))?; 

    class_name
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<()> {

    let runnable: TypedRunnableModel = tract_onnx::onnx()
        .model_for_path(Path::new("models/model.onnx"))?
        .into_optimized()?
        .into_runnable()?;

    let state = ModelState(Arc::new(Mutex::new(runnable)));
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            run_inference 
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    Ok(())
}