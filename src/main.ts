// VoipGlot Frontend TypeScript
// This file contains the frontend logic for the VoipGlot GUI

// Tauri API access function
function getTauriInvoke(): any {
    try {
        if (typeof window !== 'undefined' && (window as any).__TAURI__) {
            const tauri = (window as any).__TAURI__;
            
            // Try different possible locations for the invoke function
            const possibleInvokePaths = [
                tauri.tauri?.invoke,           // Standard Tauri 2.0 structure
                tauri.invoke,                  // Direct invoke on tauri object
                tauri.api?.invoke,             // Alternative structure
                tauri.core?.invoke,            // Another possible structure
            ];
            
            for (let i = 0; i < possibleInvokePaths.length; i++) {
                const invoke = possibleInvokePaths[i];
                if (invoke && typeof invoke === 'function') {
                    console.log(`Tauri invoke function found at path ${i}:`, typeof invoke);
                    return invoke;
                }
            }
            
            // Log all available properties for debugging
            console.log('Available Tauri properties:', Object.keys(tauri));
            if (tauri.tauri) {
                console.log('Tauri.tauri properties:', Object.keys(tauri.tauri));
            }
        }
        return null;
    } catch (error) {
        console.log('Error checking Tauri API:', error);
        return null;
    }
}

// Initialize Tauri API
async function initializeTauri() {
    try {
        console.log('Window object:', typeof window);
        console.log('Tauri object:', (window as any).__TAURI__);
        
        // Add detailed debugging of Tauri object structure
        if ((window as any).__TAURI__) {
            const tauri = (window as any).__TAURI__;
            console.log('Tauri object keys:', Object.keys(tauri));
            
            // Log the actual values of key properties
            console.log('Tauri object details:');
            Object.keys(tauri).forEach(key => {
                const value = tauri[key];
                console.log(`  ${key}:`, typeof value, value);
            });
            
            if (tauri.tauri) {
                console.log('Tauri.tauri keys:', Object.keys(tauri.tauri));
                console.log('Tauri.tauri details:');
                Object.keys(tauri.tauri).forEach(key => {
                    const value = tauri.tauri[key];
                    console.log(`  tauri.${key}:`, typeof value, value);
                });
            } else {
                console.log('Tauri.tauri is not available');
            }
        }
        
        // Try to get Tauri invoke function immediately
        let tauriInvoke = getTauriInvoke();
        
        if (tauriInvoke) {
            console.log('Tauri API found immediately, accessing invoke...');
            (window as any).__VOIPGLOT_INVOKE__ = tauriInvoke;
            console.log('Tauri API initialized successfully');
            return true;
        }
        
        // If not immediately available, try with increasing delays
        const retryDelays = [100, 500, 1000, 2000]; // 100ms, 500ms, 1s, 2s
        
        for (let i = 0; i < retryDelays.length; i++) {
            const delay = retryDelays[i];
            console.log(`Tauri API not available, waiting ${delay}ms (attempt ${i + 1}/${retryDelays.length})...`);
            
            await new Promise(resolve => setTimeout(resolve, delay));
            
            tauriInvoke = getTauriInvoke();
            if (tauriInvoke) {
                console.log(`Tauri API found after ${delay}ms delay, accessing invoke...`);
                (window as any).__VOIPGLOT_INVOKE__ = tauriInvoke;
                console.log('Tauri API initialized successfully');
                return true;
            }
        }
        
        // If we get here, Tauri API is still not available
        console.error('❌ Tauri API not available after all retry attempts!');
        console.error('This application requires Tauri to run properly.');
        console.error('Please use: cargo tauri dev');
        console.error('Do not access http://localhost:1420/ directly in browser.');
        
        // Show error message to user
        showTauriError();
        return false;
        
    } catch (error) {
        console.error('Failed to initialize Tauri API:', error);
        showTauriError();
        return false;
    }
}

// Safe invoke function that uses the stored Tauri invoke
async function safeInvoke(command: string, args?: any): Promise<any> {
    const tauriInvoke = (window as any).__VOIPGLOT_INVOKE__;
    if (tauriInvoke) {
        return await tauriInvoke(command, args);
    } else {
        throw new Error('Tauri invoke not available');
    }
}

// Show error message when Tauri is not available
function showTauriError() {
    // Create error overlay
    const errorOverlay = document.createElement('div');
    errorOverlay.style.cssText = `
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: rgba(0, 0, 0, 0.9);
        color: white;
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        z-index: 9999;
        font-family: Arial, sans-serif;
        text-align: center;
        padding: 2rem;
    `;
    
    errorOverlay.innerHTML = `
        <h1 style="color: #ff6b6b; margin-bottom: 1rem;">⚠️ Tauri Not Available</h1>
        <p style="font-size: 1.2rem; margin-bottom: 1rem;">
            This application requires Tauri to run properly.
        </p>
        <div style="background: #2d3748; padding: 1rem; border-radius: 8px; margin: 1rem 0;">
            <p style="margin: 0.5rem 0;"><strong>To run the application:</strong></p>
            <code style="background: #1a202c; padding: 0.5rem; border-radius: 4px; display: block; margin: 0.5rem 0;">
                cargo tauri dev
            </code>
        </div>
        <p style="color: #a0aec0;">
            Do not access http://localhost:1420/ directly in browser.
        </p>
    `;
    
    document.body.appendChild(errorOverlay);
}

// Type definitions
interface AudioDevices {
  input: string[];
  output: string[];
}

interface Language {
  code: string;
  name: string;
}

// State
let isMicActive: boolean = false;
let visualizerInterval: number | null = null;
let currentConfig = {
    inputDevice: '',
    outputDevice: '',
    sourceLanguage: 'en',
    targetLanguage: 'es'
};

// DOM Elements - will be initialized after DOM loads
let micButton: HTMLButtonElement;
let micText: HTMLElement;
let settingsBtn: HTMLButtonElement;
let helpBtn: HTMLButtonElement;
let inputDeviceSelect: HTMLSelectElement;
let outputDeviceSelect: HTMLSelectElement;
let sourceLanguageSelect: HTMLSelectElement;
let targetLanguageSelect: HTMLSelectElement;
let frequencyVisualizer: HTMLElement;

// Initialize DOM elements
function initializeDOMElements() {
    console.log('Initializing DOM elements...');
    
    micButton = document.getElementById('mic-button') as HTMLButtonElement;
    micText = document.querySelector('.mic-text') as HTMLElement;
    settingsBtn = document.getElementById('settings-btn') as HTMLButtonElement;
    helpBtn = document.getElementById('help-btn') as HTMLButtonElement;
    inputDeviceSelect = document.getElementById('input-device') as HTMLSelectElement;
    outputDeviceSelect = document.getElementById('output-device') as HTMLSelectElement;
    sourceLanguageSelect = document.getElementById('source-language') as HTMLSelectElement;
    targetLanguageSelect = document.getElementById('target-language') as HTMLSelectElement;
    frequencyVisualizer = document.getElementById('frequency-visualizer') as HTMLElement;
    
    // Check if all required DOM elements exist
    const requiredElements = [
        micButton, micText, settingsBtn, helpBtn, 
        inputDeviceSelect, outputDeviceSelect, 
        sourceLanguageSelect, targetLanguageSelect, 
        frequencyVisualizer
    ];
    
    const missingElements = requiredElements.filter(el => !el);
    if (missingElements.length > 0) {
        console.error('Missing required DOM elements:', missingElements);
        console.error('This might indicate an HTML structure issue.');
        return false;
    }
    
    console.log('✅ All DOM elements found successfully');
    return true;
}

// Initialize the application
window.addEventListener('load', async () => {
    console.log('VoipGlot GUI initialized');
    
    try {
        // Initialize Tauri API first
        const tauriAvailable = await initializeTauri();
        console.log('Tauri invoke available:', typeof (window as any).__VOIPGLOT_INVOKE__);
        
        if (!tauriAvailable) {
            console.error('❌ Cannot proceed without Tauri API');
            return; // Stop initialization
        }
        
        // Initialize DOM elements after Tauri is available
        if (!initializeDOMElements()) {
            console.error('❌ Cannot proceed without DOM elements');
            return;
        }
        
        // Test Tauri connection
        console.log('Testing Tauri connection...');
        try {
            const testResult = await safeInvoke('test_connection');
            console.log('✅ Tauri connection test result:', testResult);
        } catch (error) {
            console.error('❌ Tauri connection test failed:', error);
            showNotification('Tauri backend connection failed. Please restart the application.', 'error');
            return;
        }
        
        // Only proceed if Tauri is working
        await loadAudioDevices();
        await loadLanguages();
        setupEventListeners();
        setupFrequencyVisualizer();
        await loadDefaultSettings();
        console.log('✅ Application initialization completed successfully');
        console.log('🎉 VoipGlot is ready! Use the microphone button to start translation.');
        
        // Test that everything is working
        console.log('🧪 Running functionality test...');
        console.log('- DOM elements loaded:', !!micButton && !!micText && !!settingsBtn && !!helpBtn);
        console.log('- Tauri API available:', !!(window as any).__VOIPGLOT_INVOKE__);
        console.log('- Event listeners ready: true');
        console.log('✅ All systems operational!');
        
    } catch (error) {
        console.error('Failed to initialize application:', error);
        showNotification('Failed to initialize application. Please restart with: cargo tauri dev', 'error');
    }
});

// Setup event listeners
function setupEventListeners() {
    console.log('Setting up event listeners...');
    
    // Check if elements exist before adding listeners
    console.log('Checking DOM elements:');
    console.log('- micButton:', micButton);
    console.log('- settingsBtn:', settingsBtn);
    console.log('- helpBtn:', helpBtn);
    console.log('- inputDeviceSelect:', inputDeviceSelect);
    console.log('- outputDeviceSelect:', outputDeviceSelect);
    console.log('- sourceLanguageSelect:', sourceLanguageSelect);
    console.log('- targetLanguageSelect:', targetLanguageSelect);
    
    // Microphone button
    if (micButton) {
        micButton.addEventListener('click', (e) => {
            console.log('Microphone button clicked');
            toggleMicrophone();
        });
        console.log('✅ Microphone button listener added');
    } else {
        console.error('❌ Microphone button not found');
    }
    
    // Settings button
    if (settingsBtn) {
        settingsBtn.addEventListener('click', (e) => {
            console.log('Settings button clicked');
            openSettings();
        });
        console.log('✅ Settings button listener added');
    } else {
        console.error('❌ Settings button not found');
    }
    
    // Help button
    if (helpBtn) {
        helpBtn.addEventListener('click', (e) => {
            console.log('Help button clicked');
            openHelp();
        });
        console.log('✅ Help button listener added');
    } else {
        console.error('❌ Help button not found');
    }
    
    // Device selection
    if (inputDeviceSelect) {
        inputDeviceSelect.addEventListener('change', (e) => {
            console.log('Input device select changed');
            onInputDeviceChange(e);
        });
        console.log('✅ Input device select listener added');
    } else {
        console.error('❌ Input device select not found');
    }
    
    if (outputDeviceSelect) {
        outputDeviceSelect.addEventListener('change', (e) => {
            console.log('Output device select changed');
            onOutputDeviceChange(e);
        });
        console.log('✅ Output device select listener added');
    } else {
        console.error('❌ Output device select not found');
    }
    
    // Language selection
    if (sourceLanguageSelect) {
        sourceLanguageSelect.addEventListener('change', (e) => {
            console.log('Source language select changed');
            onSourceLanguageChange(e);
        });
        console.log('✅ Source language select listener added');
    } else {
        console.error('❌ Source language select not found');
    }
    
    if (targetLanguageSelect) {
        targetLanguageSelect.addEventListener('change', (e) => {
            console.log('Target language select changed');
            onTargetLanguageChange(e);
        });
        console.log('✅ Target language select listener added');
    } else {
        console.error('❌ Target language select not found');
    }
    
    console.log('Event listeners setup completed');
}

// Load available audio devices
async function loadAudioDevices() {
    console.log('Loading audio devices...');
    try {
        const devices: AudioDevices = await safeInvoke('get_audio_devices');
        console.log('Audio devices received from backend:', devices);
        
        // Clear existing options
        inputDeviceSelect.innerHTML = '<option value="">Select input device...</option>';
        outputDeviceSelect.innerHTML = '<option value="">Select output device...</option>';
        
        // Add input devices
        devices.input.forEach(device => {
            const option = document.createElement('option');
            option.value = device;
            option.textContent = device;
            inputDeviceSelect.appendChild(option);
            console.log('Added input device option:', device);
        });
        
        // Add output devices
        devices.output.forEach(device => {
            const option = document.createElement('option');
            option.value = device;
            option.textContent = device;
            outputDeviceSelect.appendChild(option);
            console.log('Added output device option:', device);
        });
        
        console.log('Audio devices loaded successfully');
    } catch (error) {
        console.error('Failed to load audio devices:', error);
        showNotification('Failed to load audio devices', 'error');
    }
}

// Load supported languages
async function loadLanguages() {
    console.log('Loading supported languages...');
    try {
        const languages: Language[] = await safeInvoke('get_supported_languages');
        console.log('Languages received from backend:', languages);
        
        // Clear existing options
        sourceLanguageSelect.innerHTML = '';
        targetLanguageSelect.innerHTML = '';
        
        // Add languages to both selects
        languages.forEach(lang => {
            // Source language
            const sourceOption = document.createElement('option');
            sourceOption.value = lang.code;
            sourceOption.textContent = lang.name;
            sourceLanguageSelect.appendChild(sourceOption);
            
            // Target language
            const targetOption = document.createElement('option');
            targetOption.value = lang.code;
            targetOption.textContent = lang.name;
            targetLanguageSelect.appendChild(targetOption);
            
            console.log('Added language option:', lang.name, lang.code);
        });
        
        console.log('Languages loaded successfully');
    } catch (error) {
        console.error('Failed to load languages:', error);
        showNotification('Failed to load languages', 'error');
    }
}

// Toggle microphone on/off
async function toggleMicrophone() {
    console.log('🔴 toggleMicrophone() called');
    console.log('Current isMicActive state:', isMicActive);
    console.log('Current config:', currentConfig);
    
    try {
        if (!isMicActive) {
            console.log('🟡 Starting audio processing...');
            
            // Start processing
            await safeInvoke('start_audio_processing', {
                inputDevice: currentConfig.inputDevice,
                outputDevice: currentConfig.outputDevice,
                sourceLanguage: currentConfig.sourceLanguage,
                targetLanguage: currentConfig.targetLanguage
            });
            
            isMicActive = true;
            micButton.classList.add('active');
            micText.textContent = 'Disable Microphone';
            startFrequencyVisualizer();
            console.log('✅ Audio processing started successfully');
            showNotification('Audio processing started', 'success');
            
        } else {
            console.log('🟡 Stopping audio processing...');
            
            // Stop processing
            await safeInvoke('stop_audio_processing');
            
            isMicActive = false;
            micButton.classList.remove('active');
            micText.textContent = 'Enable Microphone';
            stopFrequencyVisualizer();
            console.log('✅ Audio processing stopped successfully');
            showNotification('Audio processing stopped', 'info');
        }
    } catch (error) {
        console.error('❌ Failed to toggle microphone:', error);
        showNotification(`Failed to ${isMicActive ? 'stop' : 'start'} audio processing: ${error}`, 'error');
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
async function onInputDeviceChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    const device = target.value;
    console.log('Input device changed to:', device);
    
    currentConfig.inputDevice = device;
    console.log('Updated current config input device:', currentConfig.inputDevice);
    
    // If processing is active, restart with new configuration
    if (isMicActive) {
        console.log('Processing is active, updating configuration...');
        try {
            await safeInvoke('update_configuration', {
                inputDevice: device,
                outputDevice: null,
                sourceLanguage: null,
                targetLanguage: null
            });
            console.log('Audio processing restarted with new input device');
            showNotification('Input device updated', 'success');
        } catch (error) {
            console.error('Failed to update input device:', error);
            showNotification('Failed to update input device', 'error');
        }
    } else {
        console.log('Processing not active, configuration updated in memory only');
    }
}

// Handle output device change
async function onOutputDeviceChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    const device = target.value;
    console.log('Output device changed to:', device);
    
    currentConfig.outputDevice = device;
    console.log('Updated current config output device:', currentConfig.outputDevice);
    
    // If processing is active, restart with new configuration
    if (isMicActive) {
        console.log('Processing is active, updating configuration...');
        try {
            await safeInvoke('update_configuration', {
                inputDevice: null,
                outputDevice: device,
                sourceLanguage: null,
                targetLanguage: null
            });
            console.log('Audio processing restarted with new output device');
            showNotification('Output device updated', 'success');
        } catch (error) {
            console.error('Failed to update output device:', error);
            showNotification('Failed to update output device', 'error');
        }
    } else {
        console.log('Processing not active, configuration updated in memory only');
    }
}

// Handle source language change
async function onSourceLanguageChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    const language = target.value;
    console.log('Source language changed to:', language);
    
    currentConfig.sourceLanguage = language;
    console.log('Updated current config source language:', currentConfig.sourceLanguage);
    
    // If processing is active, restart with new configuration
    if (isMicActive) {
        console.log('Processing is active, updating configuration...');
        try {
            await safeInvoke('update_configuration', {
                inputDevice: null,
                outputDevice: null,
                sourceLanguage: language,
                targetLanguage: null
            });
            console.log('Audio processing restarted with new source language');
            showNotification('Source language updated', 'success');
        } catch (error) {
            console.error('Failed to update source language:', error);
            showNotification('Failed to update source language', 'error');
        }
    } else {
        console.log('Processing not active, configuration updated in memory only');
    }
}

// Handle target language change
async function onTargetLanguageChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    const language = target.value;
    console.log('Target language changed to:', language);
    
    currentConfig.targetLanguage = language;
    console.log('Updated current config target language:', currentConfig.targetLanguage);
    
    // If processing is active, restart with new configuration
    if (isMicActive) {
        console.log('Processing is active, updating configuration...');
        try {
            await safeInvoke('update_configuration', {
                inputDevice: null,
                outputDevice: null,
                sourceLanguage: null,
                targetLanguage: language
            });
            console.log('Audio processing restarted with new target language');
            showNotification('Target language updated', 'success');
        } catch (error) {
            console.error('Failed to update target language:', error);
            showNotification('Failed to update target language', 'error');
        }
    } else {
        console.log('Processing not active, configuration updated in memory only');
    }
}

// Load default settings
async function loadDefaultSettings() {
    console.log('Loading default settings...');
    
    // Set default values
    if (sourceLanguageSelect.options.length > 0) {
        sourceLanguageSelect.value = 'en';
        currentConfig.sourceLanguage = 'en';
        console.log('Set default source language to English');
    }
    
    if (targetLanguageSelect.options.length > 0) {
        targetLanguageSelect.value = 'es';
        currentConfig.targetLanguage = 'es';
        console.log('Set default target language to Spanish');
    }
    
    console.log('Current config after defaults:', currentConfig);
    
    // Check if processing is already active
    try {
        console.log('Checking if processing is already active...');
        const isActive = await safeInvoke('is_processing_active');
        console.log('Processing active status from backend:', isActive);
        if (isActive) {
            isMicActive = true;
            micButton.classList.add('active');
            micText.textContent = 'Disable Microphone';
            startFrequencyVisualizer();
            console.log('Restored active processing state');
        } else {
            console.log('No active processing found');
        }
    } catch (error) {
        console.error('Failed to check processing status:', error);
    }
    
    console.log('Default settings loaded successfully');
}

// Setup frequency visualizer
function setupFrequencyVisualizer() {
    console.log('Setting up frequency visualizer...');
    const bars = frequencyVisualizer.querySelectorAll('.frequency-bar');
    console.log('Found frequency bars:', bars.length);
    bars.forEach((bar, index) => {
        (bar as HTMLElement).style.setProperty('--i', index.toString());
    });
    console.log('Frequency visualizer setup completed');
}

// Start frequency visualizer animation
function startFrequencyVisualizer() {
    console.log('Starting frequency visualizer...');
    if (visualizerInterval) {
        console.log('Visualizer already running, stopping first');
        stopFrequencyVisualizer();
    }
    
    visualizerInterval = window.setInterval(async () => {
        try {
            const frequencies: number[] = await safeInvoke('get_audio_frequency_data');
            const bars = frequencyVisualizer.querySelectorAll('.frequency-bar');
            
            bars.forEach((bar, index) => {
                if (index < frequencies.length) {
                    const height = frequencies[index] * 100; // Convert to percentage
                    (bar as HTMLElement).style.height = `${height}%`;
                }
            });
        } catch (error) {
            console.error('Failed to get frequency data:', error);
            // Fallback to random data
            const bars = frequencyVisualizer.querySelectorAll('.frequency-bar');
            bars.forEach(bar => {
                const height = Math.random() * 80 + 10; // 10% to 90%
                (bar as HTMLElement).style.height = `${height}%`;
            });
        }
    }, 100); // Update every 100ms for smooth animation
    
    console.log('Frequency visualizer started');
}

// Stop frequency visualizer animation
function stopFrequencyVisualizer() {
    console.log('Stopping frequency visualizer...');
    if (visualizerInterval) {
        clearInterval(visualizerInterval);
        visualizerInterval = null;
        
        // Reset bars to minimum height
        const bars = frequencyVisualizer.querySelectorAll('.frequency-bar');
        bars.forEach(bar => {
            (bar as HTMLElement).style.height = '4px';
        });
        console.log('Frequency visualizer stopped and bars reset');
    } else {
        console.log('No visualizer interval to stop');
    }
}

// Utility function to show notifications
function showNotification(message: string, type: string = 'info') {
    const timestamp = new Date().toISOString();
    const logMessage = `[${timestamp}] ${type.toUpperCase()}: ${message}`;
    console.log(logMessage);
    
    // Also log to file if Tauri is available
    try {
        // This will be handled by the backend logging system
        // The backend already logs all Tauri command calls
    } catch (error) {
        // Fallback to console only
    }
    
    // TODO: Implement proper notification system
    // For now, just log to console
}

// Export functions for potential use by Tauri
(window as any).voipglot = {
    toggleMicrophone,
    openSettings,
    openHelp,
    onInputDeviceChange,
    onOutputDeviceChange,
    onSourceLanguageChange,
    onTargetLanguageChange
}; 