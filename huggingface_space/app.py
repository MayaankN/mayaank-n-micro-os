"""
Mayaank N – Micro-OS Scheduler Simulator (Hugging Face Companion)
Student Name: Mayaank N
USN: 25BTRCL148
Course: Operating Systems CA6 Project
Specialization: Scheduler Engineering – Round Robin with Dynamic Quantum
"""

import streamlit as st
import pandas as pd
import altair as alt
from collections import deque

st.set_page_config(
    page_title="Mayaank N – Micro-OS Scheduler Simulator",
    page_icon="⚙️",
    layout="wide",
)

# Header & Identification
st.title("⚙️ Mayaank N – Micro-OS Scheduler Simulator")
st.caption("Companion Interactive Simulator for Bare-Metal x86_64 Rust Microkernel (CA6 Engineering Project)")

st.warning(
    "⚠️ **EDUCATIONAL SIMULATOR NOTICE:** This application is a high-level scheduler simulation and telemetry companion. "
    "It demonstrates the mathematical principles of Round Robin and Dynamic Quantum scheduling. "
    "It is **not** the bare-metal kernel itself (the bare-metal kernel runs in QEMU via Limine bootloader on x86_64)."
)

# Student Metadata Card
with st.expander("🎓 Student & Project Information", expanded=True):
    col_a, col_b, col_c, col_d = st.columns(4)
    col_a.metric("Student Name", "Mayaank N")
    col_b.metric("USN", "25BTRCL148")
    col_c.metric("Architecture", "x86_64 Bare-Metal")
    col_d.metric("Bootloader", "Limine v8")

# Sidebar Controls
st.sidebar.header("🔧 Scheduler Configuration")

base_quantum = st.sidebar.slider("Base Time Quantum (Q_base ticks)", min_value=1, max_value=10, value=4, step=1)
min_quantum = st.sidebar.slider("Minimum Quantum (Q_min ticks)", min_value=1, max_value=4, value=2, step=1)
max_quantum = st.sidebar.slider("Maximum Quantum (Q_max ticks)", min_value=6, max_value=20, value=12, step=1)

st.sidebar.subheader("Task Workload Presets")
preset = st.sidebar.selectbox(
    "Select Workload Scenario",
    ["Default Kernel Tasks (Compute, Telemetry, Burst)", "Heavy CPU-bound Workload", "Interactive / Yielding Workload", "Mixed Heterogeneous Workload"]
)

if preset == "Default Kernel Tasks (Compute, Telemetry, Burst)":
    tasks_data = [
        {"id": "Task-1", "name": "Compute-Worker", "arrival": 0, "burst": 18, "type": "CPU-Bound"},
        {"id": "Task-2", "name": "Telemetry-Worker", "arrival": 0, "burst": 12, "type": "Balanced"},
        {"id": "Task-3", "name": "Burst-Worker", "arrival": 0, "burst": 6, "type": "Interactive/Yielding"},
    ]
elif preset == "Heavy CPU-bound Workload":
    tasks_data = [
        {"id": "Task-1", "name": "Matrix-Multiply", "arrival": 0, "burst": 25, "type": "CPU-Bound"},
        {"id": "Task-2", "name": "FFT-Calculation", "arrival": 1, "burst": 20, "type": "CPU-Bound"},
        {"id": "Task-3", "name": "Crypto-Hasher", "arrival": 2, "burst": 30, "type": "CPU-Bound"},
    ]
elif preset == "Interactive / Yielding Workload":
    tasks_data = [
        {"id": "Task-1", "name": "Keyboard-Poller", "arrival": 0, "burst": 4, "type": "Interactive"},
        {"id": "Task-2", "name": "UI-Renderer", "arrival": 1, "burst": 6, "type": "Interactive"},
        {"id": "Task-3", "name": "Packet-Dispatcher", "arrival": 2, "burst": 5, "type": "Interactive"},
    ]
else:
    tasks_data = [
        {"id": "Task-1", "name": "Database-Query", "arrival": 0, "burst": 15, "type": "CPU-Bound"},
        {"id": "Task-2", "name": "IPC-Listener", "arrival": 0, "burst": 4, "type": "Interactive"},
        {"id": "Task-3", "name": "Log-Flusher", "arrival": 2, "burst": 8, "type": "Balanced"},
        {"id": "Task-4", "name": "Physics-Engine", "arrival": 4, "burst": 22, "type": "CPU-Bound"},
    ]

# Simulation Engine Functions
def simulate_scheduler(tasks_config, base_q, min_q, max_q, is_dynamic=True):
    # Prepare task state
    tasks = []
    for t in tasks_config:
        tasks.append({
            "id": t["id"],
            "name": t["name"],
            "arrival": t["arrival"],
            "burst": t["burst"],
            "remaining": t["burst"],
            "assigned_q": base_q,
            "burst_history": [base_q],
            "first_run": None,
            "completion": None,
            "wait_time": 0,
            "turnaround": 0,
        })
    
    current_time = 0
    ready_queue = deque()
    timeline = []
    context_switches = 0
    completed = 0
    num_tasks = len(tasks)
    
    # Enqueue initial arrivals
    for i, t in enumerate(tasks):
        if t["arrival"] == 0:
            ready_queue.append(i)
            
    while completed < num_tasks:
        if not ready_queue:
            # Idle CPU step
            timeline.append({"Task": "Idle", "Start": current_time, "End": current_time + 1, "Quantum": 1})
            current_time += 1
            for i, t in enumerate(tasks):
                if t["arrival"] == current_time and t["remaining"] > 0 and i not in ready_queue:
                    ready_queue.append(i)
            continue
            
        task_idx = ready_queue.popleft()
        t = tasks[task_idx]
        
        if t["first_run"] is None:
            t["first_run"] = current_time
            
        # Determine slice
        if is_dynamic:
            # Dynamic quantum algorithm
            avg_burst = sum(t["burst_history"]) / len(t["burst_history"])
            q = t["assigned_q"]
            if avg_burst >= t["assigned_q"]:
                q = min(max_q, q + 1)
            elif avg_burst <= min_q:
                q = max(min_q, q - 1)
            if len(ready_queue) > 3:
                q = max(min_q, q - 1)
            slice_time = min(q, t["remaining"])
            t["assigned_q"] = q
        else:
            q = base_q
            slice_time = min(base_q, t["remaining"])
            
        start_time = current_time
        current_time += slice_time
        t["remaining"] -= slice_time
        t["burst_history"].append(slice_time)
        if len(t["burst_history"]) > 4:
            t["burst_history"].pop(0)
            
        timeline.append({
            "Task": f"{t['id']} ({t['name']})",
            "Start": start_time,
            "End": current_time,
            "Quantum": q,
            "Executed": slice_time,
        })
        
        # Check newly arrived tasks during execution
        for i, other in enumerate(tasks):
            if other["arrival"] > start_time and other["arrival"] <= current_time and other["remaining"] > 0 and i not in ready_queue and i != task_idx:
                ready_queue.append(i)
                
        if t["remaining"] == 0:
            t["completion"] = current_time
            t["turnaround"] = t["completion"] - t["arrival"]
            t["wait_time"] = t["turnaround"] - t["burst"]
            completed += 1
        else:
            ready_queue.append(task_idx)
            context_switches += 1
            
    avg_wait = sum(t["wait_time"] for t in tasks) / num_tasks
    avg_tat = sum(t["turnaround"] for t in tasks) / num_tasks
    
    return {
        "tasks": tasks,
        "timeline": timeline,
        "context_switches": context_switches,
        "avg_wait": avg_wait,
        "avg_tat": avg_tat,
        "total_time": current_time,
    }

# Run both simulations for comparison
res_fixed = simulate_scheduler(tasks_data, base_quantum, min_quantum, max_quantum, is_dynamic=False)
res_dynamic = simulate_scheduler(tasks_data, base_quantum, min_quantum, max_quantum, is_dynamic=True)

# Metrics Display
st.subheader("📊 Comparative Performance Telemetry")
m1, m2, m3, m4 = st.columns(4)

cs_diff = res_fixed["context_switches"] - res_dynamic["context_switches"]
wait_diff = res_fixed["avg_wait"] - res_dynamic["avg_wait"]
tat_diff = res_fixed["avg_tat"] - res_dynamic["avg_tat"]

m1.metric("Dynamic Context Switches", res_dynamic["context_switches"], delta=f"-{cs_diff}" if cs_diff > 0 else f"{abs(cs_diff)}")
m2.metric("Fixed RR Context Switches", res_fixed["context_switches"])
m3.metric("Avg Waiting Time (Dynamic)", f"{res_dynamic['avg_wait']:.2f} ticks", delta=f"-{wait_diff:.2f}" if wait_diff > 0 else f"{abs(wait_diff):.2f}")
m4.metric("Avg Turnaround Time", f"{res_dynamic['avg_tat']:.2f} ticks", delta=f"-{tat_diff:.2f}" if tat_diff > 0 else f"{abs(tat_diff):.2f}")

# Timeline Visualizer
st.subheader("📅 Dynamic Quantum Scheduling Timeline (Gantt Chart)")
df_dyn_timeline = pd.DataFrame(res_dynamic["timeline"])

chart = alt.Chart(df_dyn_timeline).mark_bar(cornerRadius=4).encode(
    x=alt.X("Start:Q", title="Simulation Time (Ticks)"),
    x2="End:Q",
    y=alt.Y("Task:N", title="Task", sort=None),
    color=alt.Color("Task:N", legend=None),
    tooltip=["Task", "Start", "End", "Executed", "Quantum"]
).properties(height=240, width="container")

st.altair_chart(chart, use_container_width=True)

# Fixed Quantum Timeline
with st.expander("🔍 View Baseline Standard Round Robin Timeline (Fixed Quantum)", expanded=False):
    df_fixed_timeline = pd.DataFrame(res_fixed["timeline"])
    chart_fixed = alt.Chart(df_fixed_timeline).mark_bar(cornerRadius=4).encode(
        x=alt.X("Start:Q", title="Simulation Time (Ticks)"),
        x2="End:Q",
        y=alt.Y("Task:N", title="Task", sort=None),
        color=alt.Color("Task:N", legend=None),
        tooltip=["Task", "Start", "End", "Executed", "Quantum"]
    ).properties(height=240, width="container")
    st.altair_chart(chart_fixed, use_container_width=True)

# Detailed Task Table
st.subheader("📋 Per-Task Scheduling Results")
tab1, tab2 = st.tabs(["Dynamic Quantum Scheduling", "Fixed Quantum Scheduling"])

with tab1:
    dyn_rows = []
    for t in res_dynamic["tasks"]:
        dyn_rows.append({
            "Task ID": t["id"],
            "Task Name": t["name"],
            "Arrival": t["arrival"],
            "Burst": t["burst"],
            "Adapted Quantum": t["assigned_q"],
            "Wait Time": t["wait_time"],
            "Turnaround Time": t["turnaround"],
        })
    st.dataframe(pd.DataFrame(dyn_rows), use_container_width=True)

with tab2:
    fixed_rows = []
    for t in res_fixed["tasks"]:
        fixed_rows.append({
            "Task ID": t["id"],
            "Task Name": t["name"],
            "Arrival": t["arrival"],
            "Burst": t["burst"],
            "Fixed Quantum": base_quantum,
            "Wait Time": t["wait_time"],
            "Turnaround Time": t["turnaround"],
        })
    st.dataframe(pd.DataFrame(fixed_rows), use_container_width=True)

# Algorithm Explanation Section
st.subheader("🧠 Mathematical Specialization: Dynamic Time Quantum Adaptation")
st.markdown(r"""
The dynamic quantum algorithm adapts each task's allocated time slice $Q_{dyn}$ according to its historical CPU burst execution and the real-time load of the ready queue:

$$Q_{dyn}(T) = \text{clamp}\left( Q_{base} + \text{sgn}(\text{burst}_{prev} - Q_{curr}) \cdot \Delta - \beta \cdot \max(0, |Q_{ready}| - 3), Q_{min}, Q_{max} \right)$$

1. **CPU-bound Expansion:** If a task continuously exhausts its assigned quantum ($\text{burst}_{avg} \ge Q$), its slice is increased ($+1$) up to $Q_{max}$ (12 ticks). This reduces context-switch overhead and memory cache invalidation.
2. **Interactive/Yielding Compression:** If a task yields early ($\text{burst}_{avg} \le Q_{min}$), its slice is shrunk to $Q_{min}$ (2 ticks), prioritizing snappy response and lower waiting latency.
3. **Queue Pressure Relief:** When the ready queue depth exceeds 3 tasks, a load penalty reduces slices to prevent latency degradation for waiting tasks.
""")

st.markdown("---")
st.caption("Mayaank N – 25BTRCL148 | Department of Computer Science & Engineering | CA6 Micro-OS Project")
