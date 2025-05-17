# Kiwi 🥝

**Kiwi** is a tiny, cross-platform image guesser app built with Tauri. It classifies images instantly right on your desktop using MobileNet ONNX.

## Features

-   **Lightweight:** Minimal resource usage thanks to Tauri and tract.
    
-   **Offline image recognition:** Uses the MobileNet ONNX model to classify images locally, no internet connection required.
    
## Installation 
### From pre-built releases

1.  Download the latest release for your OS from the [Releases](https://github.com/yourusername/kiwi/releases) page.
    
2.  Run the installer or unzip and launch the app.

### From source

Make sure you have Rust and Node.js installed.

git clone https://github.com/dan-lund/kiwi.git  
cd kiwi  
npm install  
npm run tauri build
