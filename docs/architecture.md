# Architecture

## System Architecture

```mermaid
graph TB
    subgraph "Audio Input"
        MIC[Real Microphone]
    end
    
    subgraph "VoipGlot Core"
        AM[AudioManager]
        AC[AudioCapture]
        AP[AudioPlayback]
        AProc[AudioProcessor]
        TR[Translator]
    end
    
    subgraph "Translation Pipeline"
        STT[Speech-to-Text]
        TRANS[Translation API]
        TTS[Text-to-Speech]
    end
    
    subgraph "Audio Output"
        VM[Virtual Microphone<br/>VB-CABLE]
        APP[Target Application<br/>Game/Discord/Zoom]
    end
    
    MIC --> AC
    AC --> AM
    AM --> AProc
    AProc --> TR
    TR --> STT
    STT --> TRANS
    TRANS --> TTS
    TTS --> AP
    AP --> VM
    VM --> APP
    
    style MIC fill:#e1f5fe
    style VM fill:#e1f5fe
    style APP fill:#e1f5fe
    style AM fill:#fff3e0
    style TR fill:#fff3e0
    style STT fill:#f3e5f5
    style TRANS fill:#f3e5f5
    style TTS fill:#f3e5f5
```

## Audio Pipeline Flow

```mermaid
flowchart LR
    subgraph "1. Audio Capture"
        A1[Microphone Input]
        A2[Audio Buffer]
        A3[Chunk Processing]
    end
    
    subgraph "2. Preprocessing"
        B1[Noise Reduction]
        B2[Silence Detection]
        B3[Audio Normalization]
    end
    
    subgraph "3. Speech Recognition"
        C1[STT Processing]
        C2[Text Extraction]
    end
    
    subgraph "4. Translation"
        D1[Text Translation]
        D2[Language Conversion]
    end
    
    subgraph "5. Speech Synthesis"
        E1[TTS Processing]
        E2[Audio Generation]
    end
    
    subgraph "6. Audio Output"
        F1[Audio Buffer]
        F2[Virtual Microphone]
        F3[Target Application]
    end
    
    A1 --> A2 --> A3
    A3 --> B1 --> B2 --> B3
    B3 --> C1 --> C2
    C2 --> D1 --> D2
    D2 --> E1 --> E2
    E2 --> F1 --> F2 --> F3
    
    style A1 fill:#e3f2fd
    style F3 fill:#e8f5e8
    style C1 fill:#fff3e0
    style D1 fill:#fff3e0
    style E1 fill:#fff3e0
```

## Component Interaction Diagram

```mermaid
sequenceDiagram
    participant User
    participant AudioCapture
    participant AudioManager
    participant AudioProcessor
    participant Translator
    participant STT
    participant TranslationAPI
    participant TTS
    participant AudioPlayback
    participant VirtualMic
    
    User->>AudioCapture: Speak into microphone
    AudioCapture->>AudioManager: Send audio chunks
    AudioManager->>AudioProcessor: Process audio data
    
    alt Audio contains speech
        AudioProcessor->>Translator: Request translation
        Translator->>STT: Convert speech to text
        STT-->>Translator: Return transcribed text
        Translator->>TranslationAPI: Translate text
        TranslationAPI-->>Translator: Return translated text
        Translator->>TTS: Generate speech
        TTS-->>Translator: Return audio data
        Translator-->>AudioProcessor: Return translated audio
        AudioProcessor->>AudioPlayback: Send audio for output
        AudioPlayback->>VirtualMic: Output translated speech
        VirtualMic-->>User: Play translated audio
    else Audio is silence
        AudioProcessor->>AudioProcessor: Skip processing
    end
```

## Core Components
- **AudioManager**: Orchestrates audio capture and playback
- **AudioCapture**: Handles real microphone input
- **AudioPlayback**: Outputs to virtual microphone
- **AudioProcessor**: Manages the translation pipeline
- **Translator**: Coordinates STT, translation, and TTS

## Audio Pipeline
1. **Capture**: Real-time audio from microphone
2. **Preprocessing**: Noise reduction, silence detection
3. **STT**: Convert speech to text
4. **Translation**: Translate text to target language
5. **TTS**: Convert translated text to speech
6. **Playback**: Output to virtual microphone

## Project Structure
```
voipglot-win/
├── src/
│   ├── main.rs              # Application entry point
│   ├── error.rs             # Error handling
│   ├── config.rs            # Configuration management
│   ├── audio/               # Audio processing modules
│   │   ├── mod.rs
│   │   ├── capture.rs       # Audio capture
│   │   ├── playback.rs      # Audio playback
│   │   └── processing.rs    # Audio processing pipeline
│   └── translation/         # AI translation modules
│       ├── mod.rs
│       ├── stt.rs           # Speech-to-text
│       ├── translator_api.rs # Text translation
│       └── tts.rs           # Text-to-speech
├── Cargo.toml               # Rust dependencies
├── config.toml              # Configuration file
└── README.md               # This file
```

## Data Flow Architecture

```mermaid
graph TD
    subgraph "Input Layer"
        I1[Real Microphone]
        I2[Audio Device Detection]
    end
    
    subgraph "Processing Layer"
        P1[Audio Buffer Management]
        P2[Chunk Processing]
        P3[Silence Detection]
    end
    
    subgraph "AI Services Layer"
        AI1[Speech Recognition]
        AI2[Text Translation]
        AI3[Speech Synthesis]
    end
    
    subgraph "Output Layer"
        O1[Audio Buffer]
        O2[Virtual Microphone]
        O3[Target Applications]
    end
    
    I1 --> I2
    I2 --> P1
    P1 --> P2
    P2 --> P3
    P3 --> AI1
    AI1 --> AI2
    AI2 --> AI3
    AI3 --> O1
    O1 --> O2
    O2 --> O3
    
    style I1 fill:#e8f5e8
    style O3 fill:#e8f5e8
    style AI1 fill:#fff3e0
    style AI2 fill:#fff3e0
    style AI3 fill:#fff3e0
``` 