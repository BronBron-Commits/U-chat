import sounddevice as sd
import soundfile as sf
import numpy as np
import torchaudio
import torch
import time
import os
import shutil
from speechbrain.inference import SpeakerRecognition

# === Formatting ===
GREEN = "\033[92m"
RED = "\033[91m"
CYAN = "\033[96m"
BOLD = "\033[1m"
RESET = "\033[0m"

def beep():
    try:
        os.system("paplay /usr/share/sounds/alsa/Front_Center.wav 2>/dev/null || printf '\\a'")
    except:
        pass

def print_banner():
    print(f"{CYAN}{BOLD}")
    print("╔══════════════════════════════════════════════════╗")
    print("║               UNHIDRA VOICE ENROLL              ║")
    print("╚══════════════════════════════════════════════════╝")
    print(f"{RESET}")

# === Prompts ===
prompts = [
    "Okay, let's get to it. Unhidra CLI — open it up. Time to get some work done.",
    "Unhidra CLI. That’s the command. Recognize my voice. Only me.",
    "Come on, Unhidra CLI — don’t make me say it again. Just open already.",
    "Unhidra CLI… run quietly this time. No alerts. Just background mode.",
    "Unhidra CLI now — don’t wait, just launch the terminal instantly.",
    "Every time I say ‘Unhidra CLI,’ it should respond. This is my machine. My voice. My trigger.",
    "Unhidra CLI. No delays. Terminal, front and center."
]

samplerate = 16000
duration = 3
model = SpeakerRecognition.from_hparams(source="speechbrain/spkrec-ecapa-voxceleb", savedir="tmp_spkrec")
embeddings = []

os.system("clear")
print_banner()
print(f"{CYAN}🔐 Welcome, Initiate. Let's give Unhidra your voice.\n")
time.sleep(1)

profile_name = input(f"{CYAN}🖋️  Enter a name for this voice profile (e.g. 'bronson'): {RESET}").strip()
if not profile_name:
    profile_name = "my_voiceprint"
outfile = f"{profile_name}.npy"

print(f"\n{CYAN}🎙 You'll speak 7 training phrases. Each will be recorded and analyzed.{RESET}\n")
time.sleep(1)

for i, line in enumerate(prompts, start=1):
    print(f"{CYAN}📜 Line {i}/7:{RESET}\n{BOLD}“{line}”{RESET}")
    input(f"{CYAN}   Press Enter to begin...{RESET}")
    print(f"   🎬 Recording in:", end="", flush=True)

    for c in [3, 2, 1]:
        print(f" {c}...", end="", flush=True)
        time.sleep(0.7)
    print()
    beep()
    print(f"   🎤 Speak now!", flush=True)

    audio = sd.rec(int(duration * samplerate), samplerate=samplerate, channels=1, dtype='float32')
    sd.wait()
    fname = f"voice_{i:02}.wav"
    sf.write(fname, audio, samplerate)
    print(f"   ✅ Saved: {fname}\n")

    signal, fs = torchaudio.load(fname)
    emb = model.encode_batch(signal).squeeze().detach().numpy()
    embeddings.append(emb)

# Final embedding
avg_embedding = np.mean(np.stack(embeddings), axis=0)
np.save(outfile, avg_embedding)
print(f"{GREEN}✅ Voice profile saved as {outfile}{RESET}")

# Test phase
print(f"\n{CYAN}🔍 Let's test it. Say: 'Unhidra CLI'{RESET}")
input(f"{CYAN}   Press Enter when ready...{RESET}")
beep()
audio = sd.rec(int(duration * samplerate), samplerate=samplerate, channels=1, dtype='float32')
sd.wait()
sf.write("test_clip.wav", audio, samplerate)

signal, fs = torchaudio.load("test_clip.wav")
embed = model.encode_batch(signal).squeeze().detach().numpy()
score = np.dot(embed, avg_embedding) / (np.linalg.norm(embed) * np.linalg.norm(avg_embedding))

print(f"\n{CYAN}🎤 Voice match score: {BOLD}{round(score, 3)}{RESET}")
if score > 0.55:
    print(f"{GREEN}✅ Voice match confirmed. You are known to Unhidra.{RESET}")
else:
    print(f"{RED}⛔ Voice mismatch. Consider re-training or adjusting sensitivity.{RESET}")

# Auto-patch unhidra_listen.py
target = os.path.expanduser("~/unhidra_listen.py")
if os.path.exists(target):
    with open(target, "r") as f:
        lines = f.readlines()
    with open(target, "w") as f:
        for line in lines:
            if "np.load(" in line:
                f.write(f'reference = np.load("{outfile}")\n')
            else:
                f.write(line)
    print(f"\n{CYAN}🧬 Auto-patched {target} to use your new profile.{RESET}")
else:
    print(f"\n{RED}⚠ Could not locate unhidra_listen.py to auto-patch.{RESET}")

print(f"\n{BOLD}✨ Unhidra enrollment complete. You may now invoke with confidence.{RESET}\n")
