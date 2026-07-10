use std::process::Command;
use std::thread;

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemSpeaker;

impl SystemSpeaker {
    pub fn speak(&self, text: &str) -> Result<(), String> {
        if text.trim().is_empty() {
            return Err("cannot speak empty text".to_owned());
        }

        speak_platform(text.to_owned())
    }
}

#[cfg(target_os = "windows")]
fn speak_platform(text: String) -> Result<(), String> {
    let escaped = text.replace('\'', "''");
    let script = format!(
        "Add-Type -AssemblyName System.Speech; \
         $voice = New-Object System.Speech.Synthesis.SpeechSynthesizer; \
         $voice.Rate = -1; \
         $voice.Speak('{escaped}');"
    );

    thread::Builder::new()
        .name("lexipath-speech".to_owned())
        .spawn(move || {
            let _ = Command::new("powershell.exe")
                .arg("-NoProfile")
                .arg("-NonInteractive")
                .arg("-WindowStyle")
                .arg("Hidden")
                .arg("-Command")
                .arg(script)
                .status();
        })
        .map(|_| ())
        .map_err(|error| format!("failed to start Windows speech: {error}"))
}

#[cfg(not(target_os = "windows"))]
fn speak_platform(_text: String) -> Result<(), String> {
    Err("the initial speech backend is available on Windows only".to_owned())
}
