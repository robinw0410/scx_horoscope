// SPDX-License-Identifier: GPL-2.0
//
// scx_horoscope - Astrological CPU Scheduler
//
// An experimental sched_ext scheduler that makes scheduling decisions based on
// real-time planetary positions, zodiac signs, and astrological principles.

mod astrology;

mod bpf_skel;
pub use bpf_skel::*;
pub mod bpf_intf;

#[rustfmt::skip]
mod bpf;
use bpf::{BpfScheduler, DispatchedTask, RL_CPU_ANY};

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use libbpf_rs::OpenObject;
use log::{info, debug, error};
use scx_utils::libbpf_clap_opts::LibbpfOpts;
use scx_utils::UserExitInfo;
use simplelog::{Config, LevelFilter, TermLogger, TerminalMode, ColorChoice};
use std::mem::MaybeUninit;
use std::time::SystemTime;

use astrology::AstrologicalScheduler;

/// An astrological `sched_ext` scheduler
#[derive(Debug, Clone, Parser)]
#[allow(clippy::struct_excessive_bools)]
struct Opts {
    /// Time slice duration for tasks in microseconds
    #[clap(short = 's', long, default_value = "5000")]
    slice_us: u64,

    /// Minimum time slice in microseconds
    #[clap(long, default_value = "500")]
    slice_us_min: u64,

    /// Enable verbose logging
    #[clap(short = 'v', long)]
    verbose: bool,

    /// Show cosmic weather report on startup
    #[clap(short = 'w', long)]
    cosmic_weather: bool,

    /// Update planetary positions every N seconds
    #[clap(short = 'u', long, default_value = "60")]
    update_interval: u64,

    /// Print scheduling decisions for debugging
    #[clap(short = 'd', long)]
    debug_decisions: bool,

    /// Disable retrograde effects (boring mode)
    #[clap(long)]
    no_retrograde: bool,

    /// Use 13-sign zodiac with Ophiuchus (IAU constellation boundaries)
    /// By default, uses traditional 12-sign tropical zodiac
    #[clap(long)]
    ophiuchus: bool,

    /// Enable natal birth chart per process (reads /proc/<pid>/stat creation time)
    #[clap(long)]
    birth_charts: bool,

    /// Enable planetary aspect calculations (conjunctions, trines, oppositions, etc.)
    #[clap(long)]
    aspects: bool,

    /// Show horoscope completion time predictions in debug output
    #[clap(long)]
    predictions: bool,

    /// Enable per-CPU elemental affinity (Fire/Earth/Air/Water CPUs prefer matching tasks)
    #[clap(long)]
    cpu_affinity: bool,

    /// Show the full system daily horoscope on startup (requires --cosmic-weather or -w)
    #[clap(long)]
    daily_horoscope: bool,
}

struct Scheduler<'a> {
    bpf: BpfScheduler<'a>,
    astro: AstrologicalScheduler,
    opts: Opts,
    last_update: u64,
}

impl<'a> Scheduler<'a> {
    fn init(open_object: &'a mut MaybeUninit<OpenObject>, opts: Opts) -> Result<Self> {
        let open_opts = LibbpfOpts::default();
        let slice_ns = opts.slice_us * 1000; // Convert to nanoseconds

        let bpf = BpfScheduler::init(
            open_object,
            open_opts.clone().into_bpf_open_opts(),
            0,            // exit_dump_len
            false,        // partial
            opts.verbose, // debugt
            true,         // builtin_idle
            slice_ns,     // default time slice
            "horoscope",  // scx ops name
        )?;

        #[allow(clippy::cast_possible_wrap)]
        let astro = AstrologicalScheduler::with_full_options(
            opts.update_interval as i64,
            opts.ophiuchus,
            opts.birth_charts,
            opts.aspects,
            opts.predictions,
            opts.cpu_affinity,
        );
        let last_update = Self::now();

        Ok(Self { bpf, astro, opts, last_update })
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn print_cosmic_weather(&mut self) {
        let now = Utc::now();
        let weather = self.astro.get_cosmic_weather(now);
        println!("\n{weather}\n");
    }

    fn print_daily_horoscope(&mut self) {
        let now = Utc::now();
        let horoscope = self.astro.get_daily_horoscope(now);
        println!("\n{horoscope}\n");
    }

    fn dispatch_tasks(&mut self) {
        let now_chrono = Utc::now();

        // Update planetary positions and evict stale birth charts periodically
        let current_time = Self::now();
        if current_time - self.last_update >= self.opts.update_interval {
            debug!("Updating planetary positions...");
            self.astro.evict_dead_processes();
            self.last_update = current_time;
        }

        // Process each waiting task
        loop {
            match self.bpf.dequeue_task() {
                Ok(Some(task)) => {
                    // Get task name from comm field - convert i8 to u8
                    #[allow(clippy::cast_sign_loss)]
                    let comm_bytes: Vec<u8> = task.comm.iter().map(|&c| c as u8).collect();
                    let comm = String::from_utf8_lossy(&comm_bytes)
                        .trim_end_matches('\0')
                        .to_string();

                    // Make astrological scheduling decision
                    let decision = self.astro.schedule_task(&comm, task.pid, now_chrono);

                    // Create dispatched task
                    let mut dispatched_task = DispatchedTask::new(&task);

                    // Select CPU — optionally pick the most astrologically compatible one
                    let cpu = self.bpf.select_cpu(task.pid, task.cpu, task.flags);
                    dispatched_task.cpu = if cpu >= 0 { cpu } else { RL_CPU_ANY };

                    // Per-CPU elemental affinity: log the cosmic compatibility score.
                    if self.opts.cpu_affinity && self.opts.debug_decisions && dispatched_task.cpu != RL_CPU_ANY {
                        let task_type = self.astro.classify_for_affinity(&comm);
                        let snap = self.astro.current_positions(now_chrono);
                        let score = self.astro.cpu_affinity_score(dispatched_task.cpu, task_type, &snap);
                        if (score - 1.0).abs() > 0.01 {
                            debug!("  🖥️  CPU {} affinity score for {}: {:.2}x", dispatched_task.cpu, task_type.name(), score);
                        }
                    }

                    // Calculate time slice based on priority
                    // Higher astrological priority = longer time slice
                    let priority_factor = (f64::from(decision.priority) / 1000.0).clamp(0.1, 1.0);
                    #[allow(clippy::cast_precision_loss)]
                    let base_slice = (self.opts.slice_us * 1000) as f64; // to nanoseconds
                    #[allow(clippy::cast_precision_loss)]
                    let min_slice = (self.opts.slice_us_min * 1000) as f64;

                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let slice_ns = (min_slice + (base_slice - min_slice) * priority_factor) as u64;
                    dispatched_task.slice_ns = slice_ns;

                    // Apply retrograde penalty if enabled
                    if !self.opts.no_retrograde && decision.planetary_influence < 0.0 {
                        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        let penalized = (dispatched_task.slice_ns as f64 * 0.5) as u64;
                        dispatched_task.slice_ns = penalized;
                    }

                    if self.opts.debug_decisions {
                        let slice_microseconds = dispatched_task.slice_ns / 1000;
                        debug!(
                            "[PID {}] {} | Priority: {} | Slice: {slice_microseconds}μs | {}",
                            task.pid,
                            comm,
                            decision.priority,
                            decision.reasoning
                        );

                        // Horoscope prediction for this task type
                        if self.opts.predictions {
                            let task_type = self.astro.classify_for_affinity(&comm);
                            let snap = self.astro.current_positions(now_chrono);
                            if let Some(pred) = self.astro.horoscope_prediction(&snap, task_type, now_chrono) {
                                debug!("  🔮 Forecast: {} | Power hour: {:02}:00 | Lucky: {}",
                                    pred.completion_estimate.cosmic_reason,
                                    pred.power_hour,
                                    pred.lucky_number);
                            }
                        }
                    }

                    // Dispatch the task
                    if let Err(e) = self.bpf.dispatch_task(&dispatched_task) {
                        let pid = task.pid;
                        error!("Failed to dispatch task {pid}: {e:?}");
                    }
                }
                Ok(None) => {
                    // Queue empty, exit loop normally
                    break;
                }
                Err(code) => {
                    log::error!("Failed to dequeue task from ring buffer: error code {code}");
                    break;
                }
            }
        }

        // Notify completion and sleep until more tasks arrive
        self.bpf.notify_complete(0);
    }

    fn print_stats(&mut self) {
        let nr_user_dispatches = *self.bpf.nr_user_dispatches_mut();
        let nr_kernel_dispatches = *self.bpf.nr_kernel_dispatches_mut();
        let nr_queued = *self.bpf.nr_queued_mut();
        let nr_scheduled = *self.bpf.nr_scheduled_mut();

        info!(
            "⭐ Dispatches: user={nr_user_dispatches} kernel={nr_kernel_dispatches} | Tasks: queued={nr_queued} scheduled={nr_scheduled}"
        );
    }

    fn run(&mut self) -> Result<UserExitInfo> {
        let mut prev_ts = Self::now();

        info!("🌟 Horoscope Scheduler Starting 🌟");
        info!("The cosmos shall guide your CPU scheduling decisions!");

        if self.opts.cosmic_weather {
            self.print_cosmic_weather();
        }

        if self.opts.daily_horoscope {
            self.print_daily_horoscope();
        }

        info!("Scheduler configuration:");
        info!("  Default time slice: {}μs", self.opts.slice_us);
        info!("  Min time slice: {}μs", self.opts.slice_us_min);
        info!("  Planetary update interval: {}s", self.opts.update_interval);
        info!("  Retrograde effects: {}", if self.opts.no_retrograde { "DISABLED" } else { "ENABLED" });
        info!("  Zodiac system: {}", if self.opts.ophiuchus { "13-sign (with Ophiuchus)" } else { "Traditional 12-sign" });
        info!("  Birth charts: {}", if self.opts.birth_charts { "ENABLED" } else { "disabled" });
        info!("  Planetary aspects: {}", if self.opts.aspects { "ENABLED" } else { "disabled" });
        info!("  Completion predictions: {}", if self.opts.predictions { "ENABLED" } else { "disabled" });
        info!("  CPU elemental affinity: {}", if self.opts.cpu_affinity { "ENABLED" } else { "disabled" });

        while !self.bpf.exited() {
            self.dispatch_tasks();

            let curr_ts = Self::now();
            if curr_ts > prev_ts {
                if self.opts.verbose {
                    self.print_stats();
                }
                prev_ts = curr_ts;
            }
        }

        info!("🌙 Scheduler shutting down gracefully...");
        self.bpf.shutdown_and_report()
    }
}

fn print_warning() {
    let warning = r"
**************************************************************************

🌌 ASTROLOGICAL SCHEDULER - COSMIC WARNING 🌌

This scheduler makes task scheduling decisions based on planetary positions,
zodiac signs, and astrological principles. While the astronomical calculations
are real and the scheduling logic is functional, using astrology to schedule
CPU tasks is:

- Scientifically dubious
- Cosmically hilarious
- Actually kind of working?
- Not recommended for production systems
- Perfect for conference talks and hackathons

If Mercury goes retrograde during your compile, don't say we didn't warn you!

**************************************************************************";

    println!("{warning}");
}

fn main() -> Result<()> {
    let opts = Opts::parse();

    // Set up logging
    let log_level = if opts.verbose || opts.debug_decisions {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    TermLogger::init(
        log_level,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    print_warning();

    // Initialize and run the scheduler
    let mut open_object = MaybeUninit::uninit();
    loop {
        let mut sched = Scheduler::init(&mut open_object, opts.clone())?;
        if !sched.run()?.should_restart() {
            break;
        }
    }

    Ok(())
}
