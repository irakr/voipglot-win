// VoipGlot Frontend TypeScript
// This file contains the frontend logic for the VoipGlot GUI

// Import Tauri APIs
const { invoke } = window.__TAURI__.tauri;

// Type definitions
interface AudioDevices {
  input: string[];
  output: string[];
}

interface Language {
  code: string;
  name: string;
}

// DOM Elements
const micButton = document.getElementById('mic-button') as HTMLButtonElement;
const micText = document.querySelector('.mic-text') as HTMLElement;
const settingsBtn = document.getElementById('settings-btn') as HTMLButtonElement;
const helpBtn = document.getElementById('help-btn') as HTMLButtonElement;
const inputDeviceSelect = document.getElementById('input-device') as HTMLSelectElement;
const outputDeviceSelect = document.getElementById('output-device') as HTMLSelectElement;
const sourceLanguageSelect = document.getElementById('source-language') as HTMLSelectElement;
const targetLanguageSelect = document.getElementById('target-language') as HTMLSelectElement;
const frequencyVisualizer = document.getElementById('frequency-visualizer') as HTMLElement;

// State
let isMicActive: boolean = false;
let visualizerInterval: number | null = null;

// Initialize the application
document.addEventListener('DOMContentLoaded', () => {
    console.log('VoipGlot GUI initialized');
    setupEventListeners();
    setupFrequencyVisualizer();
    loadDefaultSettings();
});

// Setup event listeners
function setupEventListeners() {
    // Microphone button
    micButton.addEventListener('click', toggleMicrophone);
    
    // Settings button
    settingsBtn.addEventListener('click', openSettings);
    
    // Help button
    helpBtn.addEventListener('click', openHelp);
    
    // Device selection
    inputDeviceSelect.addEventListener('change', onInputDeviceChange);
    outputDeviceSelect.addEventListener('change', onOutputDeviceChange);
    
    // Language selection
    sourceLanguageSelect.addEventListener('change', onSourceLanguageChange);
    targetLanguageSelect.addEventListener('change', onTargetLanguageChange);
}

// Toggle microphone on/off
function toggleMicrophone() {
    isMicActive = !isMicActive;
    
    if (isMicActive) {
        micButton.classList.add('active');
        micText.textContent = 'Disable Microphone';
        startFrequencyVisualizer();
        console.log('Microphone enabled');
        
        // TODO: Call Rust function to start audio processing
        // invoke('start_audio_processing', { 
        //     inputDevice: inputDeviceSelect.value,
        //     outputDevice: outputDeviceSelect.value,
        //     sourceLanguage: sourceLanguageSelect.value,
        //     targetLanguage: targetLanguageSelect.value
        // });
        
    } else {
        micButton.classList.remove('active');
        micText.textContent = 'Enable Microphone';
        stopFrequencyVisualizer();
        console.log('Microphone disabled');
        
        // TODO: Call Rust function to stop audio processing
        // invoke('stop_audio_processing');
    }
}

// Open settings dialog
function openSettings() {
    console.log('Opening settings...');
    // TODO: Implement settings dialog
    alert('Settings dialog will be implemented in the next phase');
}

// Open help dialog
function openHelp() {
    console.log('Opening help...');
    // TODO: Implement help dialog
    alert('Help dialog will be implemented in the next phase');
}

// Handle input device change
function onInputDeviceChange(event) {
    const device = event.target.value;
    console.log('Input device changed to:', device);
    
    // TODO: Call Rust function to update input device
    // invoke('update_input_device', { device });
}

// Handle output device change
function onOutputDeviceChange(event) {
    const device = event.target.value;
    console.log('Output device changed to:', device);
    
    // TODO: Call Rust function to update output device
    // invoke('update_output_device', { device });
}

// Handle source language change
function onSourceLanguageChange(event) {
    const language = event.target.value;
    console.log('Source language changed to:', language);
    
    // TODO: Call Rust function to update source language
    // invoke('update_source_language', { language });
}

// Handle target language change
function onTargetLanguageChange(event) {
    const language = event.target.value;
    console.log('Target language changed to:', language);
    
    // TODO: Call Rust function to update target language
    // invoke('update_target_language', { language });
}

// Load default settings
function loadDefaultSettings() {
    console.log('Loading default settings...');
    
    // TODO: Load settings from Rust backend
    // invoke('load_settings').then((settings) => {
    //     inputDeviceSelect.value = settings.inputDevice || '';
    //     outputDeviceSelect.value = settings.outputDevice || '';
    //     sourceLanguageSelect.value = settings.sourceLanguage || 'en';
    //     targetLanguageSelect.value = settings.targetLanguage || 'es';
    // });
}

// Setup frequency visualizer
function setupFrequencyVisualizer() {
    const bars = frequencyVisualizer.querySelectorAll('.frequency-bar');
    bars.forEach((bar, index) => {
        bar.style.setProperty('--i', index);
    });
}

// Start frequency visualizer animation
function startFrequencyVisualizer() {
    if (visualizerInterval) return;
    
    visualizerInterval = setInterval(() => {
        const bars = frequencyVisualizer.querySelectorAll('.frequency-bar');
        bars.forEach(bar => {
            // Generate random height for demo purposes
            const height = Math.random() * 80 + 10; // 10% to 90%
            bar.style.height = `${height}%`;
        });
    }, 100); // Update every 100ms for smooth animation
}

// Stop frequency visualizer animation
function stopFrequencyVisualizer() {
    if (visualizerInterval) {
        clearInterval(visualizerInterval);
        visualizerInterval = null;
        
        // Reset bars to minimum height
        const bars = frequencyVisualizer.querySelectorAll('.frequency-bar');
        bars.forEach(bar => {
            bar.style.height = '4px';
        });
    }
}

// Utility function to show notifications
function showNotification(message, type = 'info') {
    console.log(`${type.toUpperCase()}: ${message}`);
    // TODO: Implement proper notification system
}

// Export functions for potential use by Tauri
window.voipglot = {
    toggleMicrophone,
    openSettings,
    openHelp,
    onInputDeviceChange,
    onOutputDeviceChange,
    onSourceLanguageChange,
    onTargetLanguageChange
}; 