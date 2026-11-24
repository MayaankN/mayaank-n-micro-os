---
title: Mayaank N - Micro-OS Scheduler Simulator
emoji: ⚙️
colorFrom: indigo
colorTo: blue
sdk: static
pinned: false
license: mit
---

# Mayaank N – Micro-OS Scheduler Simulator (Hugging Face Companion)

**Student Name:** Mayaank N  
**Project:** Rust Micro-OS Engineering Project (CA6)  
**Specialization:** Scheduler Engineering – Round Robin with Dynamic Quantum  

> **DISCLAIMER:**  
> This application is an interactive educational **scheduler simulator and algorithmic modeling tool**, designed as a companion to the bare-metal x86_64 Rust Microkernel (`mayaank-micro-os`). This simulator is *not* the bare-metal kernel itself; it provides visual telemetry, Gantt timelines, ready queue tracking, and comparative metrics (Fixed RR vs. Dynamic Quantum RR).

---

## Features
- **Interactive Multi-Task Configuration:** Configure CPU burst, arrival time, and workload types (CPU-bound vs. I/O-yielding).
- **Ready Queue Visualizer:** Step-by-step state of the FIFO ready queue.
- **Dynamic Quantum Engine:** Live calculation of adapted time quantum based on task burst history and queue load.
- **Scheduling Timeline (Gantt Chart):** Visual execution timeline showing context switches.
- **Metrics Comparison:** Compares Waiting Time, Turnaround Time, and Context Switches between Standard RR and Dynamic RR.

## Running Locally

### 1. Free Static Space (Recommended — Instant Load)
Simply open `index.html` in any modern web browser, or serve it using Python's built-in static server:
```bash
python3 -m http.server 8000
# Open http://localhost:8000 in your browser
```

### 2. Streamlit Version (Preserved Companion)
The original Streamlit implementation is preserved in `app.py`:
```bash
pip install -r requirements.txt
streamlit run app.py
```
