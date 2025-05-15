// Import Tauri API
import { invoke } from '@tauri-apps/api/core';

// DOM Elements
const uploadArea = document.getElementById('upload-area');
const fileInput = document.getElementById('image-upload');
const selectedFile = document.getElementById('selected-file');
const fileName = document.getElementById('file-name');
const guessBtn = document.getElementById('load-model-btn');
const btnText = guessBtn.querySelector('.btn-text');
const loadingSpinner = document.getElementById('loading-spinner');
const status = document.getElementById('status');

// State
let currentFile = null;

// Event Listeners
fileInput.addEventListener('change', handleFileSelect);

// Button click handler
guessBtn.addEventListener('click', handleModelLoad);

// Functions
function handleFileSelect() {
  const file = fileInput.files[0];
  if (file) {
    currentFile = file;
    fileName.textContent = file.name;
    selectedFile.style.display = 'block';
    guessBtn.disabled = false;
    hideStatus();
    
    // Update upload area icon
    uploadArea.querySelector('.upload-icon').textContent = '✅';
  }
}

async function handleModelLoad() {
  if (!currentFile) {
    showStatus('Please select an image file.', 'error');
    return;
  }

  setLoading(true);
  showStatus('Analyzing image...', 'loading');

  try {
    const arrayBuffer = await currentFile.arrayBuffer();
    const bytes = Array.from(new Uint8Array(arrayBuffer));

    const result = await invoke('try_load_model', { imageBytes: bytes });
    console.log('✅ Result from Rust:', result);
    
    showStatus(`${result}`, 'success');
    uploadArea.querySelector('.upload-icon').textContent = '📁';
  } catch (error) {
    console.error('❌ Error:', error);
    showStatus(`Failed: ${error}`, 'error');
  } finally {
    setLoading(false);
  }
}

function setLoading(isLoading) {
  guessBtn.disabled = isLoading;
  loadingSpinner.style.display = isLoading ? 'inline-block' : 'none';
  btnText.textContent = isLoading ? 'Analyzing...' : 'Analyse Image';
}

function showStatus(message, type) {
  status.textContent = message;
  status.className = `status ${type}`;
  status.style.display = 'flex';
}

function hideStatus() {
  status.style.display = 'none';
}